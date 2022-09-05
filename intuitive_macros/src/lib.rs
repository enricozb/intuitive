use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, Ident, ItemFn, Pat, PatType};

/// Helper attribute macro for creating functional components.
///
/// # Usage
/// This macro is used when creating function components, where the name of
/// the generated component is the item in the attribute. For example,
/// ```rust
/// #[component(Root)]
/// pub fn render() {
///   let text = use_state(String::new);
///
///   let on_key = on_key! { [text]
///     KeyEvent { code: Char(c), .. } => text.update(|text| text.push(c)),
///     KeyEvent { code: Char(c), .. } => text.update(|text| text.pop()),
///     KeyEvent { code: Esc, .. } => event::quit(),
///   };
///
///   render! {
///     Centered() {
///       Section(title: "Input") {
///         Text(text: text.get())
///       }
///     }
///   }
/// }
/// ```
/// constructs a `Root` component, that can be used in a `render!` macro.
///
/// # Parameters
/// If the `render` function contains parameters, these will become parameters to the
/// generated component. These parameters can later be supplied when using the generated
/// component in a `render!` macro. The provided parameters **must** implement `Default`,
/// as the generated component derives `Default`.
///
/// # Generated Component
/// The generated component has a `new() -> component::Any` associated function that can
/// be used to create the component when passing it to `Terminal::new()`.
///
/// # Nuances
/// There are a couple of nuances with this macro:
/// - The visibility of the generated component will be the same as that of the
///   `render` function the `#[component(..)]` attribute is applied to.
/// - The return type to `render` (and even the function name itself) are completely
///   ignored. In order to keep things consistent, it's recommended that the function
///   is called `render` and the return type is left empty.
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
  let component_name: Ident = syn::parse(attr).unwrap();

  let ItemFn { vis, sig, block, .. } = syn::parse(item).unwrap();
  let params: Vec<_> = sig.inputs.iter().collect();
  let param_names: Vec<Box<Pat>> = params
    .iter()
    .map(|input| match input {
      FnArg::Receiver { .. } => panic!("receivers not allowed in functional component"),
      FnArg::Typed(PatType { pat, .. }) => pat,
    })
    .cloned()
    .collect();

  quote! {
    #[derive(Default)]
    #vis struct #component_name {
      #(#params),*
    }

    impl #component_name {
      fn new(#(#params),*) -> intuitive::components::Any {
        Self {
          #(#param_names),*
        }.into()
      }
    }

    impl intuitive::components::Component for #component_name {
      fn render(&self) -> intuitive::element::Any {
        let #component_name { #(#param_names),* } = self;

        #block
      }
    }
  }
  .into()
}
