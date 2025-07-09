/// Trait to allow setting metadata on a resource.
pub trait WithMetadata<T> {
    fn with_metadata(self, metadata: T) -> Self;
}
