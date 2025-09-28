use tempref::*;

#[cfg(test)]
mod tests {
    use std::i32;

    use super::*;

    #[test]
    fn unsync() {
        let value = vec![0; 128].into_boxed_slice();
        let workspace = unsync::Temp::new(value, |b| {
            b.fill(0);
        });
        {
            let mut guard = workspace.borrow_mut();
            guard.fill(1);
            assert_eq!(vec![1; 128].into_boxed_slice(), *guard);
        }
        assert_eq!(vec![0; 128].into_boxed_slice(), *workspace.borrow());

        {
            let _r1 = workspace.borrow();
            let _r2 = workspace.borrow();
        }
        {
            let mut guard = workspace.borrow_mut();
            guard.fill(2);
            assert_eq!(vec![2; 128].into_boxed_slice(), *guard);
            guard.reset();
            assert_eq!(vec![0; 128].into_boxed_slice(), *guard);
        }
        let new_value = vec![i32::MAX; 128].into_boxed_slice();
        workspace.swap(new_value);

        assert_eq!(vec![i32::MAX; 128].into_boxed_slice(), *workspace.borrow());
        workspace.reset();
        assert_eq!(vec![0; 128].into_boxed_slice(), *workspace.borrow());

        let old = workspace.replace(vec![1; 16].into_boxed_slice());
        assert_eq!(vec![0; 128].into_boxed_slice(), old);
        assert_eq!(vec![1; 16].into_boxed_slice(), *workspace.borrow());
        {
            let mut guard = workspace.borrow_mut();
            for n in guard.iter_mut() {
                *n *= 2;
            }
            assert_eq!(vec![2; 16].into_boxed_slice(), *guard);
        }
        assert_eq!(vec![0; 16].into_boxed_slice(), *workspace.borrow());
        let inner = workspace.into_inner();
        assert_eq!(vec![0; 16].into_boxed_slice(), inner);
    }

    #[test]
    fn rwlock() {
        let value = vec![0; 128].into_boxed_slice();
        let workspace = rwlock::Temp::new(value, |b| {
            b.fill(0);
        });
        {
            let _r1 = workspace.read().unwrap();
            let _r2 = workspace.read().unwrap();
        }
        {
            let mut guard = workspace.write().unwrap();
            guard.fill(2);
            assert_eq!(vec![2; 128].into_boxed_slice(), *guard);
            guard.reset();
            assert_eq!(vec![0; 128].into_boxed_slice(), *guard);
        }
        {
            let mut guard = workspace.write().unwrap();
            guard.fill(1);
            assert_eq!(vec![1; 128].into_boxed_slice(), *guard);
        }
        assert_eq!(vec![0; 128].into_boxed_slice(), *workspace.read().unwrap());

        let inner = workspace.into_inner().unwrap();
        assert_eq!(vec![0; 128].into_boxed_slice(), inner);
    }

    #[test]
    fn mutex() {
        let value = vec![1; 128].into_boxed_slice();
        let workspace = mutex::Temp::new(value, |b| {
            b.fill(0);
        });
        assert_eq!(vec![1; 128].into_boxed_slice(), *workspace.lock().unwrap());
        assert_eq!(vec![0; 128].into_boxed_slice(), *workspace.lock().unwrap());
        {
            let mut guard = workspace.lock().unwrap();
            guard.fill(2);
            assert_eq!(vec![2; 128].into_boxed_slice(), *guard);
            guard.reset();
            assert_eq!(vec![0; 128].into_boxed_slice(), *guard);
        }
        {
            let mut guard = workspace.lock().unwrap();
            guard.fill(1);
            assert_eq!(vec![1; 128].into_boxed_slice(), *guard);
        }
        assert_eq!(vec![0; 128].into_boxed_slice(), *workspace.lock().unwrap());

        let inner = workspace.into_inner().unwrap();
        assert_eq!(vec![0; 128].into_boxed_slice(), inner);
    }
}
