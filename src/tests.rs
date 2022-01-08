use crate::delta_debugging::{ddmin, Criteria};

pub struct TestCriteria<F>(F);

impl<F: FnMut(Vec<T>) -> bool, T: Copy> Criteria<T> for TestCriteria<F> {
    fn passes<'a, I: Iterator<Item = &'a T>>(&mut self, iter: I) -> bool
    where
        T: 'a,
    {
        self.0(iter.copied().collect())
    }
}

#[test]
fn test_delta_debugging() {
    #[inline]
    fn case<F: FnMut(Vec<usize>) -> bool>(input: &[usize], f: F) -> Vec<usize> {
        let mut out = input.to_vec();
        ddmin(&mut out, &mut TestCriteria(f));
        out
    }

    // average = 20
    let input: Vec<usize> = vec![10, 20, 10, 30, 25, 35, 0, 30];

    // the algorithm is not perfect! it yields a suboptimal yet 1-minimal result.
    // 1-minimal means there is no subset that satisfies the criteria.
    assert_eq!(
        vec![10, 30],
        case(&input, |v| v.iter().sum::<usize>() / v.len() == 20)
    );

    // even length. TODO improve this somehow
    assert_eq!(vec![10, 20], case(&input, |v| v.len() % 2 == 0));

    // two odd numbers
    assert_eq!(
        vec![25, 35],
        case(&input, |v| v.iter().filter(|&v| v % 2 == 1).count() == 2)
    );

    // there is a zero
    assert_eq!(vec![0], case(&input, |v| v.iter().any(|&v| v == 0)));
}
