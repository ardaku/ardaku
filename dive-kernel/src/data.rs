use Void;

pub(crate) extern "C" fn data_new(d: *mut Void, size: u32) -> *const Void {
	// use libc
	{
		extern "C" {
			fn realloc(d: *mut Void, bytes: usize) -> *mut Void;
		}

		unsafe { realloc(d, size as usize) }
	}
}
