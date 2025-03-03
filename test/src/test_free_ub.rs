use gdnative::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Arc;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_owner_free_ub();

    status
}

pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<Bar>();
}

#[derive(NativeClass)]
#[inherit(Node)]
#[no_constructor]
struct Bar(i64, Arc<AtomicUsize>);

#[methods]
impl Bar {
    #[export]
    fn free_is_not_ub(&mut self, owner: &Node) -> bool {
        unsafe {
            owner.assume_unique().free();
        }
        assert_eq!(42, self.0, "self should not point to garbage");
        true
    }

    #[export]
    fn set_script_is_not_ub(&mut self, owner: &Node) -> bool {
        owner.set_script(Null::null());
        assert_eq!(42, self.0, "self should not point to garbage");
        true
    }
}

impl Drop for Bar {
    fn drop(&mut self) {
        self.1.fetch_add(1, AtomicOrdering::AcqRel);
        self.0 = 0;
    }
}

fn test_owner_free_ub() -> bool {
    println!(" -- test_owner_free_ub");

    let ok = std::panic::catch_unwind(|| {
        let drop_counter = Arc::new(AtomicUsize::new(0));

        {
            let bar = Bar(42, Arc::clone(&drop_counter)).emplace();

            assert_eq!(Some(true), unsafe {
                bar.base().call("set_script_is_not_ub", &[]).to()
            });

            bar.into_base().free();
        }

        {
            let bar = Bar(42, Arc::clone(&drop_counter)).emplace();

            assert_eq!(Some(true), unsafe {
                bar.base().call("free_is_not_ub", &[]).to()
            });
        }

        // the values are eventually dropped
        assert_eq!(2, drop_counter.load(AtomicOrdering::Acquire));
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_owner_free_ub failed");
    }

    ok
}
