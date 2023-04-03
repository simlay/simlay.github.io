
fn main() {
    println!("Hello world args - {:?}", std::env::args());
    let first = 2;
    let second = 3;
    println!("add {} + {} = {}", first, second, add(first, second));
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_no_work() {
        let result = add(2, 2);
        assert_ne!(result, 4);
    }
}
