pub trait Criteria<T> {
    fn passes<'a, I: IntoIterator<Item = &'a T>>(&mut self, iter: I) -> bool
    where
        T: 'a;
}

/// inner function of `ddmin`
fn ddmin_inner<T, C: Criteria<T>>(items: &mut Vec<T>, test: &mut C, chunk_size: usize) {
    // iterator that yields index of the chunk
    fn chunks_helper<T>(
        slice: &[T],
        chunk_size: usize,
    ) -> impl Iterator<Item = (usize, &[T])> + '_ {
        let mut idx = 0;
        slice.chunks(chunk_size).map(move |it| {
            let idx_before = idx;
            idx += chunk_size;
            (idx_before, it)
        })
    }
    // Step 1: test if individual chunks pass,
    // in that case remove all other chunks
    {
        let result = chunks_helper(items, chunk_size)
            .find(|(_, chunk)| test.passes(chunk.iter()))
            .map(|(a, chunk)| (a, chunk.len()));

        // remove all other items
        if let Some((chunk_index, chunk_len)) = result {
            // remove items on the right
            if chunk_index + chunk_len < items.len() {
                items.drain(chunk_index + chunk_len..);
            }

            items.drain(..chunk_index);
            ddmin(items, test);
            return;
        }
    }
    // Step 2: test if the inverse of an individual chunk will pass.
    {
        let result = chunks_helper(items, chunk_size)
            .find(|&(index, chunk)| {
                let range = index..index + chunk.len();
                test.passes(
                    items
                        .iter()
                        .enumerate()
                        .filter_map(|(n, t)| (!range.contains(&n)).then(|| t)),
                )
            })
            .map(|(a, chunk)| (a, chunk.len()));

        if let Some((chunk_index, chunk_len)) = result {
            items.drain(chunk_index..chunk_index + chunk_len);
            ddmin(items, test);
            return;
        }
    }
    // Step 3: try to divide chunks more.
    {
        let new_chunk_size = chunk_size / 2;

        if new_chunk_size != 0 {
            ddmin_inner(items, test, new_chunk_size);
        }
    }
}

/// Use delta debugging to find the minimal set of items that passes a
/// certain criteria.
///
/// # Panics
///
/// This function may panic if the initial set does not pass the criteria.
///
/// # Reference
///
/// See https://dl.acm.org/doi/10.1145/3180155.3180236
pub fn ddmin<T, C: Criteria<T>>(items: &mut Vec<T>, test: &mut C) {
    // the test must pass for the input set.
    debug_assert!(test.passes(items.iter()));

    // catch cases where `items` cannot be divided.
    match &items[..] {
        [] => return,
        [_] if test.passes([]) => {
            items.clear();
            return;
        }
        [_] => return,
        _ => {}
    }

    let chunk_size = items.len() / 2;

    ddmin_inner(items, test, chunk_size)
}
