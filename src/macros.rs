#[macro_export(local_inner_macros)]
/// Create a **HashMap** with `specific type` from a list of key-value pairs
///
/// ## Example
///
/// ```
/// #[macro_use] extern crate maplit;
/// use std::collections::HashMap;
/// struct Foo;
/// struct Bar;
///
/// trait Zoo {}
///
/// impl Zoo for Foo {}
/// impl Zoo for Bar {}
///
/// # fn main() {
/// let map = hashmap_ex!(
///     HashMap<_, Box<dyn Zoo>>,
///     {
///         "a" => Box::new(Foo {}),
///         "b" => Box::new(Bar {}),
///     }
/// );
/// # }
/// ```
/// 
/// From https://github.com/kurikomoe/maplit/commit/c165ea30b4c6d750f933f668099ae35ca166794e
macro_rules! hashmap_ex {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap_ex!(@single $rest)),*]));

    ($t:ty, { $($key:expr => $value:expr,)+ } ) => { hashmap_ex!($t, { $($key => $value),+ }) };
    ($t:ty, { $($key:expr => $value:expr),* } ) => {
        {
            let _cap = hashmap_ex!(@count $($key),*);
            let mut _map: $t = ::std::collections::HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}