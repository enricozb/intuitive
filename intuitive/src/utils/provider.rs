/// Describes a context provider.
///
/// This trait is used to describe common behavior across different data that needs to
/// be tracked during rendering. For example, we need to track:
///   - hooks
///   - component descendants
///   - previous renders
///
/// This trait defines entry and exit requirements for these pieces of data to be tracked.
pub trait Provider {
  type Entry;
  type Context;
  type Exit;
  type Output;

  /// Enters into the context provider.
  ///
  /// This method requires the outer context, in the form of [`Self::Entry`].
  fn enter(&mut self, entry: Self::Entry) -> Self::Context;

  /// Exits the context provider.
  ///
  /// This method requires the context returned by [`Self::exit`], and any additional [`Self::Exit`] context.
  fn exit(&mut self, context: Self::Context, exit: Self::Exit) -> Self::Output;
}
