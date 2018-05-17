use Void;
use Future;

pub fn new() {
}

pub fn push(port: usize, data: &[u8]) -> Future<()> {
	Future::new((), push_)
}

pub fn pull(port: usize) -> Future<&'static [u8]> {
	Future::new(&[], push_)
}

extern "C" fn push_(_context: *mut Void) -> bool {
	false
}
