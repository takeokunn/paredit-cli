pub(super) fn child_path(parent: &[usize], child: usize) -> Vec<usize> {
    let mut path = parent.to_vec();
    path.push(child);
    path
}

pub(super) fn option_child_path(parent: &[usize], option: usize, child: usize) -> Vec<usize> {
    let mut path = parent.to_vec();
    path.push(option);
    path.push(child);
    path
}

pub(super) fn local_nickname_package_path(
    parent: &[usize],
    option: usize,
    pair: usize,
    child: usize,
) -> Vec<usize> {
    let mut path = parent.to_vec();
    path.push(option);
    path.push(pair);
    path.push(child);
    path
}
