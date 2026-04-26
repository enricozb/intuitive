//! Implementation of the `intuitive-macros` proc macros.
//!
//! This crate contains the actual parsing and code-generation logic,
//! operating on [`proc_macro2::TokenStream`] so it can be unit-tested
//! without needing a proc-macro crate context.
//!
//! Two public entry points:
//!
//! - [`expand_ui`] expands a `ui!{ ... }` invocation into a chain of inherent
//!   method calls (`Name { req: e, .. }.new().opt(v).build()`) that produces an
//!   `Element`.
//! - [`expand_component`] expands a `#[component] fn Name(..)` attribute macro
//!   into a props struct `Name`, a sibling `NameBuilder`, and inherent `impl`
//!   blocks (with `new`, `build`, and per-optional-arg builder methods). No
//!   trait is involved.
//!
//! See `macros.md` at the workspace root for the surface specification.

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
  Attribute, Expr, FnArg, Ident, ItemFn, Pat, PatType, ReturnType, Token, Type, braced,
  parse::{Parse, ParseStream},
  parse2,
  spanned::Spanned,
  token,
};

// =============================================================================
// `ui!{}` expansion
// =============================================================================

// -----------------------------------------------------------------------------
// AST
// -----------------------------------------------------------------------------

/// A single child in a `ui!{}` child list. See the surface grammar in
/// `macros.md`.
enum Child {
  Component(UiComponent),
  If(IfChild),
  For(ForChild),
  Match(MatchChild),
  /// `#expr` — splice a single `Element` value into the children
  /// list. The `expr` is either a bare identifier (`#name`) or a
  /// parenthesized arbitrary expression (`#(rust_expr)`).
  Splice(Expr),
  /// `#(expr)*` — splice every element produced by an iterator
  /// (`IntoIterator<Item = Element>`) into the children list.
  SpliceIter(Expr),
}

/// A component invocation: `Name(arg, .., .opt: v, ..) { children }`.
///
/// Arguments split into three buckets:
/// - `key: expr` is reserved: it sets the resulting Element's user-supplied key
///   (used to disambiguate siblings sharing a span, e.g. in `for` loops). Not
///   forwarded to the component as a prop.
/// - Plain `name: expr` (required arg). Lowered into a field of the `Name { ..
///   }` props-struct literal that the component's inherent `new` consumes.
/// - `.name: expr` (optional arg). Lowered into a `.name(expr)` call on the
///   resulting component value.
///
/// The leading `.` is a parser-local sigil; component definitions
/// distinguish required from optional via `#[default]` on the
/// corresponding function parameter.
struct UiComponent {
  name: Ident,
  required: Vec<RequiredArg>,
  optional: Vec<OptionalArg>,
  /// The `key: expr` argument, if present. Sets `Element::user_key`
  /// on the constructed element after `.build()`.
  user_key: Option<Expr>,
  /// `Some` iff the user wrote a `{ ... }` children block after the
  /// argument list.
  children: Option<Vec<Child>>,
}

struct RequiredArg {
  name: Ident,
  value: Expr,
}

struct OptionalArg {
  name: Ident,
  value: Expr,
}

struct IfChild {
  cond: Expr,
  then_branch: Vec<Child>,
  else_branch: Option<ElseBranch>,
}

enum ElseBranch {
  If(Box<IfChild>),
  Block(Vec<Child>),
}

struct ForChild {
  pat: Pat,
  iter: Expr,
  body: Vec<Child>,
}

struct MatchChild {
  scrutinee: Expr,
  arms: Vec<MatchArm>,
}

struct MatchArm {
  pat: Pat,
  guard: Option<Expr>,
  body: Vec<Child>,
}

// -----------------------------------------------------------------------------
// Parsing
// -----------------------------------------------------------------------------

impl Parse for Child {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    // `#` introduces a splice. Three forms, distinguished by what
    // follows the `#`:
    //   `# IDENT`      — bare-ident splice (sugar for `#(IDENT)`).
    //   `# ( EXPR )`   — expression splice.
    //   `# ( EXPR ) *` — iterator splice.
    if input.peek(Token![#]) {
      input.parse::<Token![#]>()?;
      if input.peek(syn::token::Paren) {
        let inner;
        syn::parenthesized!(inner in input);
        let expr: Expr = inner.parse()?;
        if !inner.is_empty() {
          return Err(inner.error("expected a single expression inside `#(..)`"));
        }
        if input.peek(Token![*]) {
          input.parse::<Token![*]>()?;
          return Ok(Child::SpliceIter(expr));
        }
        return Ok(Child::Splice(expr));
      }
      // Bare-ident form: `#name`.
      let ident: Ident = input.parse()?;
      let expr: Expr = syn::parse_quote!(#ident);
      return Ok(Child::Splice(expr));
    }

    let lookahead = input.lookahead1();
    if lookahead.peek(Token![if]) {
      input.parse().map(Child::If)
    } else if lookahead.peek(Token![for]) {
      input.parse().map(Child::For)
    } else if lookahead.peek(Token![match]) {
      input.parse().map(Child::Match)
    } else if lookahead.peek(Ident) {
      input.parse().map(Child::Component)
    } else {
      Err(lookahead.error())
    }
  }
}

impl Parse for UiComponent {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let name: Ident = input.parse()?;

    // `(` arg_list? `)` (optional). Each arg is either `name: expr`
    // (required) or `.name: expr` (optional). When the parenthesized
    // arg list is omitted entirely (e.g. `HStack { .. }`), the
    // component is treated as having no required, no optional, and no
    // `key:` argument — equivalent to `HStack() { .. }`. This is
    // purely a surface-syntax shortcut for components whose required
    // props are all defaulted away (`#[default]`) or absent.
    let mut required = Vec::new();
    let mut optional = Vec::new();
    let mut user_key: Option<Expr> = None;
    if input.peek(token::Paren) {
      let args_inner;
      syn::parenthesized!(args_inner in input);
      while !args_inner.is_empty() {
        let is_optional = args_inner.peek(Token![.]);
        if is_optional {
          args_inner.parse::<Token![.]>()?;
        }
        let arg_name: Ident = args_inner.parse()?;
        // If `:` follows, parse an explicit value expression.
        // Otherwise treat the name itself as the value — the
        // shorthand form `Foo(bar)` / `Foo(.bar)` is sugar for
        // `Foo(bar: bar)` / `Foo(.bar: bar)`, mirroring Rust's
        // struct field shorthand.
        let value = if args_inner.peek(Token![:]) {
          args_inner.parse::<Token![:]>()?;
          // `parse_without_eager_brace` so `Foo(x: Style) { child() }`
          // doesn't read the children brace as a struct-literal body of
          // `Style`.
          Expr::parse_without_eager_brace(&args_inner)?
        } else {
          syn::parse_quote!(#arg_name)
        };
        if !is_optional && arg_name == "key" {
          // `key: expr` is reserved — it sets `Element::user_key` and
          // is not forwarded to the component as a prop. Only the
          // non-optional form is reserved; `.key: ...` (with a
          // leading sigil) is still a regular optional argument if a
          // component author happens to define one.
          if user_key.is_some() {
            return Err(syn::Error::new(arg_name.span(), "duplicate `key:` argument"));
          }
          user_key = Some(value);
        } else if is_optional {
          optional.push(OptionalArg { name: arg_name, value });
        } else {
          required.push(RequiredArg { name: arg_name, value });
        }
        if args_inner.is_empty() {
          break;
        }
        args_inner.parse::<Token![,]>()?;
      }
    }

    // Optional `{ children }`.
    let children = if input.peek(token::Brace) {
      let body;
      braced!(body in input);
      let mut kids = Vec::new();
      while !body.is_empty() {
        kids.push(body.parse::<Child>()?);
      }
      Some(kids)
    } else {
      None
    };

    Ok(UiComponent {
      name,
      required,
      optional,
      user_key,
      children,
    })
  }
}

impl Parse for IfChild {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    input.parse::<Token![if]>()?;
    let cond = Expr::parse_without_eager_brace(input)?;

    let then_body;
    braced!(then_body in input);
    let mut then_branch = Vec::new();
    while !then_body.is_empty() {
      then_branch.push(then_body.parse::<Child>()?);
    }

    let else_branch = if input.peek(Token![else]) {
      input.parse::<Token![else]>()?;
      if input.peek(Token![if]) {
        Some(ElseBranch::If(Box::new(input.parse::<IfChild>()?)))
      } else {
        let else_body;
        braced!(else_body in input);
        let mut kids = Vec::new();
        while !else_body.is_empty() {
          kids.push(else_body.parse::<Child>()?);
        }
        Some(ElseBranch::Block(kids))
      }
    } else {
      None
    };

    Ok(IfChild {
      cond,
      then_branch,
      else_branch,
    })
  }
}

impl Parse for ForChild {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    input.parse::<Token![for]>()?;
    let pat = Pat::parse_single(input)?;
    input.parse::<Token![in]>()?;
    let iter = Expr::parse_without_eager_brace(input)?;

    let body_inner;
    braced!(body_inner in input);
    let mut body = Vec::new();
    while !body_inner.is_empty() {
      body.push(body_inner.parse::<Child>()?);
    }
    Ok(ForChild { pat, iter, body })
  }
}

impl Parse for MatchChild {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    input.parse::<Token![match]>()?;
    let scrutinee = Expr::parse_without_eager_brace(input)?;

    let arms_inner;
    braced!(arms_inner in input);

    let mut arms = Vec::new();
    while !arms_inner.is_empty() {
      arms.push(arms_inner.parse::<MatchArm>()?);
      if arms_inner.peek(Token![,]) {
        arms_inner.parse::<Token![,]>()?;
      }
    }
    Ok(MatchChild { scrutinee, arms })
  }
}

impl Parse for MatchArm {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let pat = Pat::parse_multi_with_leading_vert(input)?;
    let guard = if input.peek(Token![if]) {
      input.parse::<Token![if]>()?;
      Some(Expr::parse_without_eager_brace(input)?)
    } else {
      None
    };
    input.parse::<Token![=>]>()?;

    let body = if input.peek(token::Brace) {
      let inner;
      braced!(inner in input);
      let mut kids = Vec::new();
      while !inner.is_empty() {
        kids.push(inner.parse::<Child>()?);
      }
      kids
    } else {
      vec![input.parse::<Child>()?]
    };
    Ok(MatchArm { pat, guard, body })
  }
}

// -----------------------------------------------------------------------------
// Lowering
// -----------------------------------------------------------------------------

/// The top of `ui!{}` must be a single component. Control flow at the
/// very top has nothing to push into and would not produce an `Element`.
struct UiInput(UiComponent);

impl Parse for UiInput {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![if]) || lookahead.peek(Token![for]) || lookahead.peek(Token![match]) {
      return Err(syn::Error::new(
        input.span(),
        "the top level of `ui!{}` must be a single component invocation; \
         `if`/`for`/`match` are only allowed inside a children block",
      ));
    }
    let comp = input.parse::<UiComponent>()?;
    if !input.is_empty() {
      return Err(syn::Error::new(
        input.span(),
        "`ui!{}` accepts exactly one top-level component; \
         wrap multiple elements in a container",
      ));
    }
    Ok(UiInput(comp))
  }
}

impl UiComponent {
  /// Emit this component as an expression of type `Element`.
  ///
  /// Lowering shape (see `macros.md` for the spec):
  ///
  /// ```ignore
  /// {
  ///   let mut __element: Element = Name { req1: e1, .. }
  ///       .new()                                  // -> NameBuilder
  ///       .opt1(v1)
  ///       .opt2(v2)
  ///       .build();
  ///   __element = __element.with_span(Span::new(file!(), line!(), column!()));
  ///   __element = __element.with_user_key(Some(<key-expr>.to_string()));   // if `key:` present
  ///   __element = __element.with_children(vec![ /* pushed kids */ ]);      // if `{ ... }` present
  ///   __element
  /// }
  /// ```
  fn expand_call(&self) -> TokenStream {
    let name = &self.name;

    let props_fields = self.required.iter().map(|a| {
      let n = &a.name;
      let v = &a.value;
      quote_spanned!(n.span()=> #n: #v)
    });
    // `Name { req1: e1, .. }` — a value of the props struct `Name`.
    let props_literal = quote_spanned! {name.span()=>
      #name { #(#props_fields),* }
    };

    // Consume the props, returning a `NameBuilder` with optional
    // fields defaulted.
    let mut chain = quote_spanned! {name.span()=>
      #props_literal.new()
    };

    // Optional args -> `.foo(v)` calls.
    for opt in &self.optional {
      let n = &opt.name;
      let v = &opt.value;
      chain = quote_spanned!(n.span()=> #chain.#n(#v));
    }

    // Post-build identity-stamping. `ui!{}` is responsible for
    // setting the resulting Element's `span` (and optionally
    // `user_key`) so siblings at distinct source lines / loop
    // iterations end up with distinct `ElementKey`s when mounted.
    let span_stamp = quote_spanned! {name.span()=>
      __element = __element.with_span(::intuitive::runtime::Span::new(
        ::core::file!(), ::core::line!(), ::core::column!(),
      ));
    };
    let key_stamp = if let Some(key_expr) = &self.user_key {
      quote_spanned! {key_expr.span()=>
        __element = __element.with_user_key(::core::option::Option::Some(
          ::std::string::ToString::to_string(&(#key_expr))
        ));
      }
    } else {
      quote!()
    };

    // Children block — build a `Vec<Element>` and stamp it onto
    // the result via `with_children`. `if`/`for`/`match` inside
    // the block lower to ordinary Rust control flow that pushes
    // onto `__kids`.
    let children_stamp = if let Some(kids) = &self.children {
      let push_stmts: Vec<TokenStream> = kids.iter().map(Child::expand_push).collect();
      quote_spanned! {name.span()=>
        __element = __element.with_children({
          #[allow(clippy::vec_init_then_push)]
          {
            let mut __kids: ::std::vec::Vec<::intuitive::runtime::Element> = ::std::vec::Vec::new();
            #(#push_stmts)*
            __kids
          }
        });
      }
    } else {
      quote!()
    };

    // Wrapped in a block so `#[allow]` can sit on a `let`
    // statement, suppressing `redundant_field_names` (the
    // `Name { foo: foo }` shape is unavoidable for the
    // common `Foo(foo: foo)` idiom).
    quote_spanned! {name.span()=>
      {
        #[allow(clippy::redundant_field_names)]
        let mut __element: ::intuitive::runtime::Element = #chain.build();
        #span_stamp
        #key_stamp
        #children_stamp
        __element
      }
    }
  }
}

impl Child {
  /// Emit a statement that pushes 0 or more `Element`s onto a local
  /// `__kids: Vec<Element>` in scope.
  fn expand_push(&self) -> TokenStream {
    match self {
      Child::Component(c) => {
        let call = c.expand_call();
        quote_spanned!(c.name.span()=> __kids.push(#call);)
      }
      Child::If(if_child) => if_child.expand_push(),
      Child::For(for_child) => {
        let pat = &for_child.pat;
        let iter = &for_child.iter;
        let body: Vec<TokenStream> = for_child.body.iter().map(Child::expand_push).collect();
        quote_spanned! {iter.span()=>
          for #pat in #iter {
            #(#body)*
          }
        }
      }
      Child::Match(match_child) => {
        let scrutinee = &match_child.scrutinee;
        let arms: Vec<TokenStream> = match_child
          .arms
          .iter()
          .map(|arm| {
            let pat = &arm.pat;
            let guard = arm.guard.as_ref().map(|g| quote!(if #g));
            let body: Vec<TokenStream> = arm.body.iter().map(Child::expand_push).collect();
            quote! {
              #pat #guard => { #(#body)* }
            }
          })
          .collect();
        quote_spanned! {scrutinee.span()=>
          match #scrutinee {
            #(#arms),*
          }
        }
      }
      // `#name` / `#(expr)` — push one Element value. The
      // expression is evaluated once in the surrounding scope and
      // moved into the children vec. (If the user wants to keep a
      // copy, they write `#(name.clone())`.)
      Child::Splice(expr) => {
        quote_spanned!(expr.span()=> __kids.push(#expr);)
      }
      // `#(expr)*` — extend by an iterator of Element. The
      // expression must evaluate to `IntoIterator<Item = Element>`.
      Child::SpliceIter(expr) => {
        quote_spanned!(expr.span()=> __kids.extend(#expr);)
      }
    }
  }
}

impl IfChild {
  fn expand_push(&self) -> TokenStream {
    let cond = &self.cond;
    let then_body: Vec<TokenStream> = self.then_branch.iter().map(Child::expand_push).collect();
    let else_tail = self.else_branch.as_ref().map(|e| match e {
      ElseBranch::If(nested) => {
        let nested_ts = nested.expand_push();
        quote!(else #nested_ts)
      }
      ElseBranch::Block(kids) => {
        let body: Vec<TokenStream> = kids.iter().map(Child::expand_push).collect();
        quote!(else { #(#body)* })
      }
    });
    quote_spanned! {cond.span()=>
      if #cond {
        #(#then_body)*
      } #else_tail
    }
  }
}

/// Expand a `ui!{ ... }` invocation.
///
/// # Errors
///
/// Returns a parse error if the input does not match the `ui!{}`
/// grammar described in `macros.md`.
pub fn expand_ui(input: TokenStream) -> syn::Result<TokenStream> {
  let UiInput(component) = parse2::<UiInput>(input)?;
  Ok(component.expand_call())
}

// =============================================================================
// `#[component]` expansion
// =============================================================================

/// One parameter of a `#[component]`-annotated function.
struct ComponentParam {
  /// User-visible name. Becomes a struct field and (if optional) a
  /// builder-method name.
  name: Ident,
  ty: Type,
  kind: ParamKind,
  /// Whether the user wrote `mut` on this parameter. Preserved so
  /// that the `let Self { .. } = self` destructure in `build()`
  /// emits `mut name` and the body can mutate the local.
  mutable: bool,
}

enum ParamKind {
  /// Required: appears as a field of the props struct `Name`.
  Required,
  /// Optional: builder method emitted; default expression filled in
  /// at construction time. `None` => `Default::default()`.
  Optional { default: Option<Expr> },
  /// Special: receives the composed-children vector (the `{ ... }`
  /// block at the `ui!{}` call site). The macro materializes a
  /// fresh `Vec<Element>` at the top of the render closure by
  /// cloning `ctx.composed()`; mutating the local does not affect
  /// the runtime's stored `Element::children`, so the param is
  /// effectively "owned by the body, regenerated per render."
  ///
  /// Not a builder field and not user-supplied at the call site;
  /// `ui!{}` populates composed children via the `{ ... }` block
  /// directly on the resulting `Element` value.
  Children,
  /// Special: the render-context parameter (literally named `ctx`).
  /// When present, the user's body is wrapped in
  /// `Element::new(move |ctx: <ty>| { body })` so that callers can
  /// reference `ctx` directly instead of writing the `Element::new(...)`
  /// scaffolding themselves. A `Ctx` param does not appear on the
  /// props struct or the builder struct; it exists only to
  /// receive `&mut Context<'_>` inside the render closure.
  Ctx,
}

/// A parsed `#[component]`-annotated function definition.
struct ComponentFn {
  vis: syn::Visibility,
  name: Ident,
  params: Vec<ComponentParam>,
  /// The user-written return type, if any. `None` only in the
  /// `ContextRender` body mode, where the macro emits
  /// `-> ::intuitive::runtime::Element` itself.
  return_type: Option<Type>,
  /// How to wrap the user's body in the emitted `build()` method.
  /// Determined by the presence of a `ctx` parameter and the
  /// declared return type. See [`BodyMode`].
  body_mode: BodyMode,
  body: syn::Block,
}

/// How the user's body relates to the emitted `Element`-producing
/// `build()` method. Selected at parse time from the function's
/// signature and used at emit time to choose the body wrapping.
#[derive(Clone, Copy)]
enum BodyMode {
  /// No `ctx` parameter. The body itself produces the `Element`
  /// value returned by `build()` — typically a `ui!{ .. }`
  /// expression. The macro re-emits the body verbatim. Used for
  /// pure composition (`TasksPane`, `TimerPane`).
  ReturnsElement,
  /// `ctx` parameter present, return type declared as `Element`.
  /// The body runs effects (signal reads, `on_key`, `spawn`, etc.)
  /// and evaluates to an `Element`. The macro wraps the body in a
  /// `Box::new(move |ctx| { let elem = <body>; ctx.render(.., elem); })`
  /// render closure, so the returned element is automatically
  /// drawn into the component's own region. Used for
  /// composition-with-effects (`TasksBody`, `TimerBody`).
  ContextThenElement,
  /// `ctx` parameter present, no return type. The body *is* the
  /// render closure: it draws via `ctx.region()` and/or `ctx.render`
  /// directly. The macro wraps the body in
  /// `Box::new(move |ctx| <body>)`. Used for raw renderers
  /// (`Text`, `Border`, `HStack`, `VStack`, `List`).
  ContextRender,
}

impl ComponentFn {
  #[allow(clippy::too_many_lines)]
  fn parse(item: ItemFn) -> syn::Result<Self> {
    if item.sig.asyncness.is_some() {
      return Err(syn::Error::new(
        item.sig.asyncness.span(),
        "`#[component]` functions cannot be `async`",
      ));
    }
    if !item.sig.generics.params.is_empty() {
      return Err(syn::Error::new(
        item.sig.generics.span(),
        "`#[component]` does not (yet) support generic components",
      ));
    }
    let declared_return = match item.sig.output.clone() {
      ReturnType::Type(_, ty) => Some(*ty),
      ReturnType::Default => None,
    };

    // Snapshot the signature span before draining `inputs`, since
    // the `for arg in item.sig.inputs` loop below partially moves
    // `item.sig`.
    let sig_span = item.sig.span();
    let output_span = item.sig.output.span();
    let mut params = Vec::new();
    for arg in item.sig.inputs {
      let FnArg::Typed(PatType { attrs, pat, ty, .. }) = arg else {
        return Err(syn::Error::new(
          arg.span(),
          "`#[component]` functions cannot take `self`",
        ));
      };
      let Pat::Ident(ref pat_ident) = *pat else {
        return Err(syn::Error::new(
          pat.span(),
          "`#[component]` parameters must be plain identifiers",
        ));
      };
      let name = pat_ident.ident.clone();
      let mutable = pat_ident.mutability.is_some();

      // Classify by attributes.
      let mut kind: Option<ParamKind> = None;
      for attr in &attrs {
        if attr.path().is_ident("default") {
          if kind.is_some() {
            return Err(syn::Error::new(
              attr.span(),
              "parameter has more than one `#[component]` classifier attribute",
            ));
          }
          let default = parse_default_attr(attr)?;
          kind = Some(ParamKind::Optional { default });
        } else if attr.path().is_ident("children") {
          if kind.is_some() {
            return Err(syn::Error::new(
              attr.span(),
              "parameter has more than one `#[component]` classifier attribute",
            ));
          }
          if !matches!(attr.meta, syn::Meta::Path(_)) {
            return Err(syn::Error::new(attr.span(), "`#[children]` takes no arguments"));
          }
          kind = Some(ParamKind::Children);
        } else {
          // Unknown attribute on a `#[component]` parameter. Surface
          // it loudly rather than silently ignoring, because typoed
          // `#[defalt]` would otherwise become a required arg.
          return Err(syn::Error::new(
            attr.span(),
            "unknown attribute on `#[component]` parameter (expected `#[default]` or `#[children]`)",
          ));
        }
      }
      let kind = kind.unwrap_or_else(|| {
        // No explicit classifier: a parameter literally named
        // `ctx` is treated as the render-context parameter; any
        // other name is a Required prop.
        if name == "ctx" {
          ParamKind::Ctx
        } else {
          ParamKind::Required
        }
      });

      params.push(ComponentParam {
        name,
        ty: (*ty).clone(),
        kind,
        mutable,
      });
    }

    // At most one `ctx` param.
    let ctx_count = params.iter().filter(|p| matches!(p.kind, ParamKind::Ctx)).count();
    if ctx_count > 1 {
      return Err(syn::Error::new(
        sig_span,
        "`#[component]` accepts at most one `ctx` parameter",
      ));
    }

    // At most one `#[children]` param. A `#[children]` param
    // implies a `ctx` is also present: the macro emits
    // `let <name> = ctx.composed().to_vec()` at the top of the
    // render closure, which requires `ctx` to be in scope.
    let children_count = params.iter().filter(|p| matches!(p.kind, ParamKind::Children)).count();
    if children_count > 1 {
      return Err(syn::Error::new(
        sig_span,
        "`#[component]` accepts at most one `#[children]` parameter",
      ));
    }
    if children_count == 1 && ctx_count == 0 {
      return Err(syn::Error::new(
        sig_span,
        "`#[children]` parameter requires a `ctx` parameter \
         (the children vec is materialized from `ctx.composed()` per render)",
      ));
    }

    // Body-mode rule, driven by `ctx`-presence and the declared
    // return type:
    //
    //   no ctx    + -> Element   => ReturnsElement
    //   no ctx    + no return    => error (must return Element)
    //   has ctx   + -> Element   => ContextThenElement
    //                              (run effects, then auto-render
    //                              the returned Element into our
    //                              own region)
    //   has ctx   + no return    => ContextRender
    //                              (body IS the render closure)
    let has_ctx = ctx_count == 1;
    let (return_type, body_mode) = match (has_ctx, declared_return) {
      (false, Some(ty)) => (Some(ty), BodyMode::ReturnsElement),
      (false, None) => {
        return Err(syn::Error::new(
          output_span,
          "`#[component]` functions without a `ctx` parameter must return `Element`",
        ));
      }
      (true, None) => (None, BodyMode::ContextRender),
      (true, Some(ty)) => (Some(ty), BodyMode::ContextThenElement),
    };

    Ok(ComponentFn {
      vis: item.vis,
      name: item.sig.ident,
      params,
      return_type,
      body_mode,
      body: *item.block,
    })
  }
}

/// Parse `#[default]` or `#[default = "expr"]`.
fn parse_default_attr(attr: &Attribute) -> syn::Result<Option<Expr>> {
  match &attr.meta {
    syn::Meta::Path(_) => Ok(None),
    syn::Meta::NameValue(nv) => {
      // The RHS is a `syn::Expr` per attribute grammar. For our
      // purposes the user can write either `#[default = MY_CONST]`
      // (an expression) or `#[default = "expr_in_quotes"]` (a string
      // literal we re-parse as an expression). The latter is the
      // common form because attribute syntax restricts what
      // expressions can appear unquoted.
      if let Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s), ..
      }) = &nv.value
      {
        let parsed: Expr = s.parse()?;
        Ok(Some(parsed))
      } else {
        Ok(Some(nv.value.clone()))
      }
    }
    syn::Meta::List(_) => Err(syn::Error::new(
      attr.span(),
      "`#[default]` accepts either no value or `= \"expression\"`",
    )),
  }
}

impl ComponentFn {
  #[allow(clippy::too_many_lines)]
  fn expand(&self) -> TokenStream {
    let vis = &self.vis;
    let name = &self.name;
    let builder_name = quote::format_ident!("{}Builder", name);

    // ---- Generic type parameters for required props ---------------
    //
    // Each required parameter `req: T` becomes a generic type
    // parameter on the component props struct *named the same as
    // the parameter itself* (`req`), with default `= T`. The struct
    // carries `req: req` (field-name vs. type-name live in different
    // namespaces in Rust, so this is unambiguous). The `impl` block
    // hosting `new()` bounds `req: Into<T>` and calls `.into()` on
    // each required field when populating the builder.
    //
    // This is what lets a user write `Text(text: "hi")` (passing
    // `&'static str`) while the component body still sees `text:
    // String`. The default `= T` keeps `Name` usable as a bare type
    // identifier in user code without turbofish.
    //
    // Naming the generic after the parameter (rather than an
    // internal `__P_<name>` placeholder) makes the rustdoc output
    // read sensibly — `struct List<items, selected, ..>` instead
    // of `struct List<__P_items, __P_selected, ..>`. snake_case
    // generic param names trip `non_camel_case_types`, which is
    // suppressed on every emitted site below.
    //
    // The builder itself does NOT carry these generics: it stores
    // concrete `T`s, populated via `.into()` in `new()`. Generics
    // never leak past `new()`.
    let required_generics: Vec<(&Ident, &Type)> = self
      .params
      .iter()
      .filter_map(|p| match p.kind {
        ParamKind::Required => Some((&p.name, &p.ty)),
        _ => None,
      })
      .collect();

    // `<a = A, b = B, ..>` for the props-struct declaration
    // (defaults present). Empty if there are no required params, in
    // which case the struct is non-generic.
    let struct_generics_decl = if required_generics.is_empty() {
      quote!()
    } else {
      let decls = required_generics.iter().map(|(g, ty)| quote!(#g = #ty));
      quote!(<#(#decls),*>)
    };
    // `<a, b, ..>` for use in `impl<..>` headers (no defaults
    // allowed there).
    let impl_generics = if required_generics.is_empty() {
      quote!()
    } else {
      let gs = required_generics.iter().map(|(g, _)| quote!(#g));
      quote!(<#(#gs),*>)
    };
    // `<a, b, ..>` for use on the type in `impl ... Name<..>` /
    // `Self = Name<..>`. Same shape as `impl_generics`.
    let type_generics = impl_generics.clone();
    // `where a: Into<A>, b: Into<B>` clause, used on the impl block
    // that hosts `new()`.
    let into_where_clause = if required_generics.is_empty() {
      quote!()
    } else {
      let bounds = self.params.iter().filter_map(|p| match p.kind {
        ParamKind::Required => {
          let g = &p.name;
          let ty = &p.ty;
          Some(quote!(#g: ::core::convert::Into<#ty>))
        }
        _ => None,
      });
      quote!(where #(#bounds),*)
    };

    // Fields of the props struct `Name`: required params only.
    // Each required field is typed with the generic that shares its
    // name (`req: req`) so `Name { req: e }` infers from the
    // supplied expression and `Into<T>` does the conversion at
    // `.new()`.
    //
    // Fields carry the component's own visibility (`#vis`) so the
    // struct literal `Name { req: ... }` can be written wherever
    // the struct itself is nameable. Without this, fields would
    // default to private and `ui!{}` calls from outside the
    // defining module would not type-check.
    let props_struct_fields = self.params.iter().filter_map(|p| {
      if matches!(p.kind, ParamKind::Required) {
        let n = &p.name;
        Some(quote!(#vis #n: #n,))
      } else {
        None
      }
    });

    // Field declarations for the builder struct (all real params,
    // i.e. excluding the render-context param).
    let builder_fields = self.params.iter().filter_map(|p| {
      let n = &p.name;
      let ty = &p.ty;
      match p.kind {
        ParamKind::Required | ParamKind::Optional { .. } => Some(quote!(#n: #ty,)),
        // `#[children]` and `ctx` are not user-supplied at the call
        // site; both are populated by the macro at render time.
        ParamKind::Children | ParamKind::Ctx => None,
      }
    });

    // Names of the required params, used to destructure `Self { .. }`
    // inside `new()`.
    let required_names: Vec<_> = self
      .params
      .iter()
      .filter_map(|p| matches!(p.kind, ParamKind::Required).then_some(&p.name))
      .collect();

    // Initializers for builder fields inside `new()`. Required
    // params are forwarded from the destructured props struct
    // through `.into()` (the `<name>: Into<T>` bound on the
    // `impl` block makes this valid for any user-supplied type the
    // `Into` trait connects to `T`); optional params get their
    // declared default; the children factory starts as None. The
    // render-context param has no builder field and so contributes
    // no initializer.
    let builder_inits = self.params.iter().filter_map(|p| {
      let n = &p.name;
      match &p.kind {
        ParamKind::Required => Some(quote!(#n: ::core::convert::Into::into(#n),)),
        ParamKind::Optional { default } => {
          let init = if let Some(expr) = default {
            quote!(#expr)
          } else {
            quote!(::core::default::Default::default())
          };
          Some(quote!(#n: #init,))
        }
        ParamKind::Children | ParamKind::Ctx => None,
      }
    });

    // Builder methods: one per optional param. Optional-arg setters
    // take `impl Into<T>` and convert internally, so a user can
    // write e.g. `.title("Tasks")` even though the field is
    // `String`.
    let builder_methods = self.params.iter().filter_map(|p| {
      let n = &p.name;
      let ty = &p.ty;
      match p.kind {
        ParamKind::Required | ParamKind::Children | ParamKind::Ctx => None,
        ParamKind::Optional { .. } => Some(quote_spanned! {n.span()=>
          #[must_use]
          pub fn #n<__V>(mut self, value: __V) -> Self
          where
            __V: ::core::convert::Into<#ty>,
          {
            self.#n = ::core::convert::Into::into(value);
            self
          }
        }),
      }
    });

    // The `build()` body destructures `self` into locals named
    // after each parameter, so the user's function body can
    // reference them by the names they declared. If the user wrote
    // `mut` on the parameter, preserve it in the destructure so
    // the local is mutable inside the body.
    let destructure_fields = self.params.iter().filter_map(|p| {
      let n = &p.name;
      match p.kind {
        ParamKind::Required | ParamKind::Optional { .. } => {
          if p.mutable {
            Some(quote!(mut #n,))
          } else {
            Some(quote!(#n,))
          }
        }
        // Not builder fields, so not part of the destructure.
        ParamKind::Children | ParamKind::Ctx => None,
      }
    });

    // Prologue inserted at the top of the render closure (for
    // `ContextRender` / `ContextThenElement` modes). If there's a
    // `#[children]` parameter, materialize a fresh `Vec<Element>`
    // by cloning `ctx.composed()`; bind it as a local with the
    // user's chosen name. Mutating this local does not affect the
    // runtime-owned composed children (the Element is `Clone`, so
    // each entry is a cheap Arc-bumping clone).
    let children_prologue = self.params.iter().find_map(|p| {
      if matches!(p.kind, ParamKind::Children) {
        let n = &p.name;
        let ty = &p.ty;
        let ctx_name = self
          .params
          .iter()
          .find(|q| matches!(q.kind, ParamKind::Ctx))
          .map(|q| &q.name)
          .expect("parse() guarantees a ctx param when there's a children param");
        // The user's declared type is preserved (so writing
        // `Vec<Wrong>` is a compile error). The right-hand side
        // always produces a `Vec<Element>`; if the user's `Vec<T>`
        // doesn't match, rustc emits a type error at the
        // assignment.
        //
        // Synthetic tokens (the `let mut`, the call to
        // `ctx.composed().to_vec()`) are emitted with call-site
        // spans via plain `quote!` so that IDE "go to
        // definition" inside the user body does not return
        // results pointing at these injected tokens. Only the
        // user-written identifier `#n` and type `#ty` carry their
        // original spans (preserved by `quote!` itself, since
        // they were parsed from user input).
        Some(quote! {
          #[allow(unused_mut)]
          let mut #n: #ty = #ctx_name.composed().to_vec();
        })
      } else {
        None
      }
    });
    let body = &self.body;

    // Synthesize the outer body and return type from `body_mode`.
    //
    // - `ReturnsElement`: body is the value of `build()`; re-emit the declared
    //   return type verbatim.
    // - `ContextRender`: body IS the render closure; wrap in `Box::new(move |ctx:
    //   <ty>| <body>)` and fix the return type to `Element`.
    // - `ContextThenElement`: body produces an `Element` value after running
    //   ctx-effects; wrap in a render closure that binds the body to a local and
    //   forwards it to `ctx.render` into the component's own region.
    let ctx_param = self.params.iter().find(|p| matches!(p.kind, ParamKind::Ctx));
    let (return_type_ts, body_ts) = match self.body_mode {
      BodyMode::ReturnsElement => {
        let ty = self
          .return_type
          .as_ref()
          .expect("parse() guarantees a return type in ReturnsElement mode");
        (quote!(#ty), quote!(#body))
      }
      BodyMode::ContextRender => {
        let ctx_param = ctx_param.expect("ContextRender implies a ctx param");
        let ctx_name = &ctx_param.name;
        let ctx_ty = &ctx_param.ty;
        // No return type was declared; emit the fully-qualified path
        // so the generated `build()` compiles even if the user has
        // not imported `Element`.
        let return_ts = quote!(::intuitive::runtime::Element);
        // If there's a `#[children]` param, materialize the
        // composed-children vec into a local before running the
        // user body. The user body remains a `{ ... }` block; we
        // wrap it in an outer scoped block so the prologue and the
        // body share a scope.
        //
        // Span hygiene: the synthetic path
        // `::intuitive::runtime::Element::new` and the closure
        // boilerplate are emitted with plain `quote!` (call-site
        // spans), so IDE "go to definition" on identifiers inside
        // the user body does NOT collect these synthetic tokens
        // as matches. (When the whole wrapper carried
        // `body.span()`, clicking `region.fill` would also return
        // `Element`, `Element::new`, and the surrounding module
        // roots.) The user's `#body` and `#ctx_name`/`#ctx_ty`
        // already carry their original spans, since `quote!`
        // preserves spans of spliced syn nodes.
        //
        // We still anchor the synthetic `move` keyword to the
        // body's span via `quote_spanned!` so that closure
        // trait-bound diagnostics (e.g. "cannot move out of
        // captured variable") point at the user-written body
        // rather than the macro definition.
        let move_kw = quote_spanned!(body.span()=> move);
        let wrapped = quote! {
          ::intuitive::runtime::Element::new(#move_kw |#ctx_name: #ctx_ty| {
            #children_prologue
            #body
          })
        };
        (return_ts, wrapped)
      }
      BodyMode::ContextThenElement => {
        let ctx_param = ctx_param.expect("ContextThenElement implies a ctx param");
        let ctx_name = &ctx_param.name;
        let ctx_ty = &ctx_param.ty;
        // Forward the user's written return type verbatim so that
        // imports like `use intuitive::runtime::Element` are
        // actually used and clippy does not warn about them.
        let return_ts = self
          .return_type
          .as_ref()
          .map_or_else(|| quote!(::intuitive::runtime::Element), |ty| quote!(#ty));
        // Auto-render the user-returned Element into the
        // component's own region. The user wrote
        //     ui! { ... }
        // as the tail expression; we hand that to ctx.render (the
        // dynamic-child API) so the body of a composition-with-
        // effects component reads like a pure render returning an
        // Element. The runtime mounts/reconciles the returned
        // Element under a key derived from its own span+user_key
        // (stamped by `ui!{}` at the body's call site).
        //
        // See the `ContextRender` arm above for the span-hygiene
        // rationale. All synthetic tokens (the wrapper path, the
        // `__child`/`__rect` locals, the `ctx.render` call) use
        // call-site spans so they don't pollute IDE "go to
        // definition" results inside the user body.
        let move_kw = quote_spanned!(body.span()=> move);
        let wrapped = quote! {
          ::intuitive::runtime::Element::new(#move_kw |#ctx_name: #ctx_ty| {
            #children_prologue
            let __child: ::intuitive::runtime::Element = #body;
            let __rect = #ctx_name.region().rect();
            #ctx_name.render(__child, __rect);
          })
        };
        (return_ts, wrapped)
      }
    };

    // The component type itself is a plain struct holding the
    // required parameters. Constructing a value of this struct
    // (`Name { .. }`) is what the user writes (through `ui!{}`)
    // to provide required arguments; calling `.new()` on that
    // value yields the builder, which holds every parameter and
    // has the optional-arg setters plus the final `build()`.
    //
    // (Historical note: this used to be a single-variant
    // `enum Name { Props { .. } }`, written as a workaround for
    // unstable inherent associated types. With the surface
    // settled on `Name { .. }`, the workaround is no longer
    // needed and the struct form is both simpler and more honest
    // in rustdoc output.)
    //
    // `struct_field_names` fires when a field shares its prefix
    // with its enclosing struct (e.g. `TextBuilder { text: String }`).
    // For generated code mirroring the user's chosen field names
    // this is expected and noisy to require the user to suppress
    // at the definition site.
    //
    // The per-required-prop generic type parameters are named after
    // the user's parameter names (snake_case), which trips the
    // `non_camel_case_types` lint. Suppress on every site that
    // mentions them.
    quote! {
      #[allow(
        clippy::struct_field_names,
        non_camel_case_types,
      )]
      #vis struct #name #struct_generics_decl {
        #(#props_struct_fields)*
      }

      #[allow(clippy::struct_field_names)]
      #vis struct #builder_name {
        #(#builder_fields)*
      }

      #[allow(non_camel_case_types)]
      impl #impl_generics #name #type_generics #into_where_clause {
        /// Consume the props value, returning a fully-defaulted
        /// builder. Required props are converted via `Into<T>`
        /// here, so the user may supply any type for which `Into`
        /// connects to the canonical field type.
        #[must_use]
        #vis fn new(self) -> #builder_name {
          let Self { #(#required_names,)* } = self;
          #builder_name {
            #(#builder_inits)*
          }
        }
      }

      impl #builder_name {
        #vis fn build(self) -> #return_type_ts {
          let Self { #(#destructure_fields)* } = self;
          #body_ts
        }

        #(#builder_methods)*
      }
    }
  }
}

/// Expand `#[component] fn Name(..) -> Element { .. }` into a props
/// struct `Name`, a sibling `NameBuilder`, and inherent `impl` blocks
/// containing `new` (on `Name`) and `build` plus per-optional-arg
/// builder methods (on `NameBuilder`).
///
/// # Errors
///
/// Returns a parse error if the input is not a function, if its
/// signature is unsupported (generic, async, missing return type),
/// or if a parameter has an unknown classifier attribute.
pub fn expand_component(_attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
  let item: ItemFn = parse2(item)?;
  let component = ComponentFn::parse(item)?;
  Ok(component.expand())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  /// Normalize whitespace so token-stream comparisons are robust.
  fn norm(ts: &TokenStream) -> String {
    ts.to_string().split_whitespace().collect::<Vec<_>>().join(" ")
  }

  fn ui_ok(input: TokenStream) -> String {
    norm(&expand_ui(input).expect("expected successful ui expansion"))
  }

  /// String-based parse helper used by tests whose input contains
  /// `#(...)` splice syntax, which collides with `quote!{}`'s own
  /// repetition syntax. Parses the input as a `TokenStream` first,
  /// avoiding `quote!`.
  fn ui_ok_str(input: &str) -> String {
    use std::str::FromStr;
    let ts = TokenStream::from_str(input).expect("parse input as TokenStream");
    norm(&expand_ui(ts).expect("expected successful ui expansion"))
  }

  fn component_ok(item: TokenStream) -> String {
    norm(&expand_component(TokenStream::new(), item).expect("expected successful #[component]"))
  }

  // ---- ui!{} ----

  #[test]
  fn ui_leaf_no_args() {
    let out = ui_ok(quote!(A()));
    // `Name { ... }.new()` is the call-site shape.
    assert!(out.contains("A { }"), "props struct literal: {out}");
    assert!(out.contains(". new ()"), "new() call: {out}");
    assert!(out.contains(". build ()"), "build() call: {out}");
    assert!(!out.contains(":: Props"), "no `::Props` variant qualifier: {out}");
    assert!(!out.contains("AProps"), "no separate Props struct identifier: {out}");
  }

  #[test]
  fn ui_required_arg_goes_in_props_literal() {
    let out = ui_ok(quote!( Text(text: "hi") ));
    assert!(
      out.contains("Text { text : \"hi\" }"),
      "required arg in `Name {{ .. }}` struct literal: {out}"
    );
  }

  #[test]
  fn ui_optional_arg_is_builder_call() {
    let out = ui_ok(quote!( Text(text: "hi", .align: Align::Center) ));
    assert!(
      out.contains("Text { text : \"hi\" }"),
      "only required arg in props struct: {out}"
    );
    assert!(out.contains(". align (Align :: Center)"), "optional setter: {out}");
  }

  #[test]
  fn ui_multiple_optionals_chain() {
    let out = ui_ok(quote! {
      Text(text: "hi", .align: Align::Center, .fg: Color::Red, .bold: true)
    });
    assert!(out.contains(". align (Align :: Center)"));
    assert!(out.contains(". fg (Color :: Red)"));
    assert!(out.contains(". bold (true)"));
    // build() is the outermost call in the chain.
    assert!(out.contains(". build ()"), "inherent build: {out}");
  }

  #[test]
  fn ui_container_stamps_children_directly() {
    let out = ui_ok(quote! {
      HStack(flex: vec![1, 1]) { A() B() }
    });
    // No more factory closure; `.with_children(...)` receives a
    // `Vec<Element>` built inline.
    assert!(out.contains(". with_children ("), "with_children setter: {out}");
    assert!(!out.contains("move ||"), "no factory closure: {out}");
    assert!(out.contains("__kids . push"), "push statements: {out}");
    // Span is stamped onto each constructed element so siblings on
    // different source lines end up with distinct ElementKeys.
    assert!(out.contains(". with_span ("), "with_span call present: {out}");
  }

  #[test]
  fn ui_if_in_children() {
    let out = ui_ok(quote! {
      HStack(flex: vec![1, 1]) {
        A()
        if cond { B() } else { C() }
      }
    });
    let push_sites = out.matches("__kids . push").count();
    assert_eq!(push_sites, 3, "three component-call push sites: {out}");
    assert!(out.contains("if cond"));
  }

  #[test]
  fn ui_for_in_children() {
    let out = ui_ok(quote! {
      VStack(flex: ws) {
        for t in tasks {
          Row(name: t)
        }
      }
    });
    assert!(out.contains("for t in tasks"));
    assert!(out.contains("__kids . push"));
  }

  #[test]
  fn ui_splice_ident_pushes_directly() {
    // `#name` inside a children block pushes `name` onto the
    // children vec without intermediate construction.
    let out = ui_ok_str("HStack(flex: vec![1, 1]) { #foo }");
    assert!(out.contains("__kids . push (foo) ;"), "splice ident: {out}");
  }

  #[test]
  fn ui_splice_expr_pushes_expression() {
    let out = ui_ok_str("HStack(flex: vec![1, 1]) { #(make_one(i)) }");
    assert!(out.contains("__kids . push (make_one (i)) ;"), "splice expr: {out}");
  }

  #[test]
  fn ui_splice_iter_extends() {
    let out = ui_ok_str("HStack(flex: vec![1, 1]) { #(items.iter().map(row))* }");
    assert!(
      out.contains("__kids . extend (items . iter () . map (row)) ;"),
      "splice iter: {out}"
    );
  }

  #[test]
  fn ui_splice_mixes_with_components_and_control_flow() {
    let out = ui_ok_str(
      "VStack(flex: ws) {
        A()
        #header
        for t in tasks { Row(name: t) }
        #(footer.clone())
        #(items.iter().cloned())*
      }",
    );
    // All five distinct push/extend sites:
    assert!(out.contains("__kids . push (header)"), "ident: {out}");
    assert!(out.contains("__kids . push (footer . clone ())"), "expr: {out}");
    assert!(
      out.contains("__kids . extend (items . iter () . cloned ())"),
      "iter: {out}"
    );
  }

  #[test]
  fn ui_field_shorthand_required_arg() {
    // `Foo(bar)` is sugar for `Foo(bar: bar)`.
    let out = ui_ok(quote!(Text(text)));
    assert!(
      out.contains("Text { text : text }"),
      "shorthand expands to required field: {out}"
    );
  }

  #[test]
  fn ui_field_shorthand_optional_arg() {
    // `.foo` is sugar for `.foo: foo`.
    let out = ui_ok(quote!(Text(text: "hi", .align)));
    assert!(
      out.contains(". align (align)"),
      "shorthand expands to optional setter: {out}"
    );
  }

  #[test]
  fn ui_field_shorthand_mixed_with_explicit() {
    // Shorthand and explicit forms can be mixed freely.
    let out = ui_ok(quote!(Text(text, .align: Align::Center)));
    assert!(out.contains("Text { text : text }"), "required shorthand: {out}");
    assert!(out.contains(". align (Align :: Center)"), "explicit optional: {out}");
  }

  #[test]
  fn ui_no_parens_treated_as_empty_arg_list() {
    // `HStack { .. }` is sugar for `HStack() { .. }`: zero required
    // and zero optional args, but children are still allowed.
    let out = ui_ok(quote! {
      HStack { A() B() }
    });
    assert!(out.contains("HStack { }"), "empty props struct literal: {out}");
    assert!(out.contains(". new ()"), "new() call: {out}");
    assert!(out.contains(". with_children ("), "with_children setter: {out}");
    assert!(out.contains(". build ()"), "build() call: {out}");
  }

  #[test]
  fn ui_no_parens_leaf() {
    // `A` alone (no parens, no children) is the same as `A()`.
    let out = ui_ok(quote!(A));
    assert!(out.contains("A { }"), "empty props struct literal: {out}");
    assert!(out.contains(". new ()"));
    assert!(out.contains(". build ()"));
  }

  #[test]
  fn ui_no_parens_inside_children_block() {
    // The no-parens form must also work for child components inside
    // a parent's `{ .. }` block.
    let out = ui_ok(quote! {
      HStack { Border { Text(text: "hi") } }
    });
    // The nested `Border { .. }` is parsed as a `UiComponent` with no
    // required/optional args and one child.
    assert!(out.contains("Border { }"), "empty Border props struct: {out}");
    assert!(out.contains("Text { text : \"hi\" }"), "inner Text props: {out}");
  }

  #[test]
  fn ui_top_level_if_rejected() {
    let err = expand_ui(quote!(if x { A() } else { B() })).unwrap_err();
    assert!(err.to_string().contains("top level"));
  }

  #[test]
  fn ui_multiple_top_level_rejected() {
    let err = expand_ui(quote!( A() B() )).unwrap_err();
    assert!(err.to_string().contains("exactly one"));
  }

  // ---- #[component] ----

  #[test]
  fn component_emits_struct_and_builder_and_impls() {
    let out = component_ok(quote! {
      pub fn Text(text: String) -> Element {
        Box::new(move |_ctx| {})
      }
    });
    // Component is now a plain props struct named `Text`,
    // generic over each required prop with default `= T`. The
    // generic param is named after the original parameter
    // (`text`), not an internal placeholder.
    assert!(
      out.contains("pub struct Text < text = String >"),
      "Text props struct is generic over required prop with default: {out}"
    );
    // The struct uses the generic, not the concrete type, and
    // its fields are `pub` so the literal is constructible from
    // outside the defining module.
    assert!(
      out.contains("pub struct Text < text = String > { pub text : text , }"),
      "props struct fields are pub and typed by generic: {out}"
    );
    // Sanity: the old enum-with-Props-variant shape is gone.
    assert!(!out.contains("enum Text"), "no enum Text: {out}");
    assert!(!out.contains(":: Props"), "no ::Props qualifier: {out}");
    // Builder struct holds every parameter, with concrete types.
    assert!(out.contains("pub struct TextBuilder"), "TextBuilder struct: {out}");
    // Inherent impls on both, no trait. The `Text` impl carries
    // the generic header + `Into<T>` bound.
    assert!(
      out.contains("impl < text > Text < text > where text : :: core :: convert :: Into < String >"),
      "impl Text<text> with Into bound: {out}"
    );
    assert!(out.contains("impl TextBuilder"), "inherent impl TextBuilder: {out}");
    assert!(!out.contains("Component"), "no Component trait reference: {out}");
    // new() takes `self` (the props value) and returns the builder.
    assert!(out.contains("fn new (self) -> TextBuilder"), "new signature: {out}");
    // build() lives on the builder.
    assert!(out.contains("fn build (self)"), "build signature: {out}");
    // The required field is `.into()`-converted at `new()`.
    assert!(
      out.contains("text : :: core :: convert :: Into :: into (text)"),
      ".into() conversion in new(): {out}"
    );
  }

  #[test]
  fn component_required_field_only_on_props_struct() {
    let out = component_ok(quote! {
      pub fn Text(text: String) -> Element {
        Box::new(move |_ctx| {})
      }
    });
    // Required param lives directly on the props struct, typed
    // by its generic and `pub` so it's constructible by callers.
    assert!(
      out.contains("pub struct Text < text = String > { pub text : text , }"),
      "required on pub props struct, typed by generic: {out}"
    );
    // Builder also has it as a plain (non-pub) field; the
    // builder is not user-constructible directly.
    assert!(
      out.contains("pub struct TextBuilder { text : String , }"),
      "required also on builder (non-pub field): {out}"
    );
    // No builder method for required param.
    assert!(!out.contains("pub fn text ("), "no required-arg setter: {out}");
  }

  #[test]
  fn component_optional_field_gets_builder() {
    let out = component_ok(quote! {
      pub fn Text(
        text: String,
        #[default = "Align::Left"] align: Align,
      ) -> Element {
        Box::new(move |_ctx| {})
      }
    });
    // `align` does not appear on the props struct; it only
    // exists on the builder.
    assert!(
      !out.contains("pub struct Text < text = String > { pub text : text , pub align"),
      "optional not on props struct: {out}"
    );
    // Builder method takes `impl Into<T>` so callers can pass any
    // convertible type.
    assert!(
      out.contains(
        "pub fn align < __V > (mut self , value : __V) -> Self where __V : :: core :: convert :: Into < Align >"
      ),
      "builder setter is generic over Into<T>: {out}"
    );
    assert!(
      out.contains("self . align = :: core :: convert :: Into :: into (value)"),
      "setter converts via .into(): {out}"
    );
    // Default expression used in new().
    assert!(out.contains("align : Align :: Left ,"), "default expr in new: {out}");
  }

  #[test]
  fn component_default_without_value_uses_default_trait() {
    let out = component_ok(quote! {
      pub fn Text(
        text: String,
        #[default] fg: Color,
      ) -> Element {
        Box::new(move |_ctx| {})
      }
    });
    assert!(
      out.contains("fg : :: core :: default :: Default :: default () ,"),
      "Default::default() init: {out}"
    );
    assert!(
      out.contains(
        "pub fn fg < __V > (mut self , value : __V) -> Self where __V : :: core :: convert :: Into < Color >"
      ),
      "setter is generic over Into<Color>: {out}"
    );
  }

  #[test]
  fn component_children_param_emits_prologue() {
    // `#[children]` marks a parameter that the macro fills in at
    // render time from `ctx.composed().to_vec()`. The param is
    // NOT a builder field (not user-supplied at the call site),
    // and the closure body sees it bound as a fresh local.
    let out = component_ok(quote! {
      pub fn HStack(
        ctx: &mut Context<'_>,
        flex: Vec<Flex>,
        #[children] children: Vec<Element>,
      ) {
        let _ = (ctx, flex, children);
      }
    });
    // Not on props struct.
    assert!(
      !out.contains("pub struct HStack < flex = Vec < Flex > > { pub flex : flex , pub children"),
      "children not on props struct: {out}"
    );
    // Not in builder fields.
    assert!(
      !out.contains("pub struct HStackBuilder { flex : Vec < Flex > , children"),
      "children not a builder field: {out}"
    );
    // No builder setter for it.
    assert!(
      !out.contains("pub fn children"),
      "no builder setter for children: {out}"
    );
    // Prologue in the render closure materializes the vec.
    assert!(
      out.contains("let mut children : Vec < Element > = ctx . composed () . to_vec () ;"),
      "children prologue present: {out}"
    );
  }

  #[test]
  fn component_children_without_ctx_rejected() {
    let err = expand_component(
      TokenStream::new(),
      quote! {
        pub fn Foo(#[children] kids: Vec<Element>) -> Element { unimplemented!() }
      },
    )
    .unwrap_err();
    assert!(
      err.to_string().contains("requires a `ctx` parameter"),
      "expected ctx-required error: {err}"
    );
  }

  #[test]
  fn component_unknown_param_attr_errors() {
    let err = expand_component(
      TokenStream::new(),
      quote! {
        pub fn Text(#[defalt] text: String) -> Element { unimplemented!() }
      },
    )
    .unwrap_err();
    assert!(err.to_string().contains("unknown attribute"));
  }

  #[test]
  fn component_async_rejected() {
    let err = expand_component(
      TokenStream::new(),
      quote! {
        pub async fn Foo(x: u32) -> Element { unimplemented!() }
      },
    )
    .unwrap_err();
    assert!(err.to_string().contains("async"));
  }

  #[test]
  fn component_missing_return_type_rejected() {
    let err = expand_component(
      TokenStream::new(),
      quote! {
        pub fn Foo(x: u32) { }
      },
    )
    .unwrap_err();
    assert!(err.to_string().contains("Element"));
  }

  // ---- ctx parameter ----

  #[test]
  fn component_ctx_wraps_body_in_element_new() {
    // A component with only a `ctx` parameter and no return type:
    // the user's body should be wrapped in
    // `Element::new(move |ctx: ..|`.
    let out = component_ok(quote! {
      pub fn Foo(ctx: &mut Context<'_>) {
        let _ = ctx;
      }
    });
    // No props-struct fields (ctx is not a prop). The struct is
    // emitted as `Foo { }` (always braced) so `Foo { }` works at
    // the call site uniformly, with or without required args.
    assert!(out.contains("pub struct Foo { }"), "empty Foo props struct: {out}");
    // No builder fields (ctx is not stored on the builder).
    assert!(out.contains("pub struct FooBuilder { }"), "empty FooBuilder: {out}");
    // Build returns Element and wraps the body in Element::new.
    assert!(
      out.contains("pub fn build (self) -> :: intuitive :: runtime :: Element"),
      "build returns Element: {out}"
    );
    assert!(
      out.contains(":: intuitive :: runtime :: Element :: new (move | ctx :"),
      "body wrapped in Element::new(move |ctx: ..|): {out}"
    );
  }

  #[test]
  fn component_ctx_with_required_and_optional_args() {
    // `ctx` coexists with normal props. The render closure body
    // captures the destructured props by move; ctx never appears
    // as a prop or builder field.
    let out = component_ok(quote! {
      pub fn Foo(
        ctx: &mut Context<'_>,
        name: String,
        #[default = "false"] flag: bool,
      ) {
        let _ = (ctx, name, flag);
      }
    });
    // Required prop appears on the props struct, typed by its
    // same-named generic.
    assert!(
      out.contains("pub struct Foo < name = String > { pub name : name , }"),
      "required only on pub props struct: {out}"
    );
    // `ctx` is not a builder field.
    assert!(
      !out.contains("ctx :") || !out.contains("struct FooBuilder { ctx :"),
      "ctx not stored on builder: {out}"
    );
    // The builder still has name + flag fields, plus the
    // optional-arg setter (now `Into<T>`-generic).
    assert!(
      out.contains(
        "pub fn flag < __V > (mut self , value : __V) -> Self where __V : :: core :: convert :: Into < bool >"
      ),
      "flag setter present: {out}"
    );
    // Body is wrapped.
    assert!(
      out.contains(":: intuitive :: runtime :: Element :: new (move | ctx :"),
      "body wrapped: {out}"
    );
  }

  #[test]
  fn component_ctx_then_element_auto_renders() {
    // Composition-with-effects: `ctx` is present AND the function
    // declares `-> Element`. The macro should wrap the body in a
    // render closure that binds the returned Element to a local
    // and forwards it to `ctx.render` into the component's own
    // region.
    let out = component_ok(quote! {
      pub fn Foo(ctx: &mut Context<'_>, name: String) -> Element {
        let _ = (ctx, name);
        ui! { Bar() }
      }
    });
    // Body still goes through the Element::new(move |ctx| ..) wrap.
    assert!(
      out.contains(":: intuitive :: runtime :: Element :: new (move | ctx :"),
      "body wrapped in render closure: {out}"
    );
    // Returned element bound to a local and rendered.
    assert!(
      out.contains("let __child : :: intuitive :: runtime :: Element ="),
      "returned element captured into __child: {out}"
    );
    assert!(out.contains("ctx . render ("), "auto-emitted ctx.render call: {out}");
    // `build()` returns the user's written type verbatim, so that
    // an import like `use intuitive::runtime::Element` is actually
    // used and clippy does not warn about it.
    assert!(
      out.contains("pub fn build (self) -> Element"),
      "build returns Element: {out}"
    );
  }

  #[test]
  fn component_multiple_ctx_params_rejected() {
    let err = expand_component(
      TokenStream::new(),
      quote! {
        pub fn Foo(ctx: &mut Context<'_>, ctx: &mut Context<'_>) { }
      },
    )
    .unwrap_err();
    assert!(err.to_string().contains("at most one `ctx`"));
  }

  #[test]
  fn component_without_ctx_still_requires_return_type() {
    // Sanity check: removing the `ctx` parameter brings back the
    // old "must return Element" rule.
    let err = expand_component(
      TokenStream::new(),
      quote! {
        pub fn Foo(x: u32) { }
      },
    )
    .unwrap_err();
    assert!(err.to_string().contains("Element"));
  }
}
