use Void;

pub struct Future<T> {
	data: T,
	call: extern "C" fn(*mut Void) -> bool,
}

impl<T> Future<T> {
	/// Create a new future.  Call should return true for a yield and false
	/// for a return.
	pub fn new(data: T, call: extern "C" fn(*mut Void) -> bool)
		-> Future<T>
	{
		Future {
			data, call
		}
	}
}
