use game::santorini::*;



#[derive(Eq, Copy, PartialEq, Clone, Debug)]
pub enum EntryType {
	Exact,
	Lower,
	Upper,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct TranspositionEntry {
	pub state: State,
	pub value: HeuristicValue,
	pub entry_type: EntryType,
	pub depth: u8,
}

pub fn print_sizes() {

}

#[cfg(test)]
mod tests {
	use game::santorini::*;
	use std::mem;

    #[test]
    fn sizes() {
    	println!("State size -> {}", mem::size_of::<State>());
	    println!("EntryType size -> {}", mem::size_of::<EntryType>());
	    println!("TranspositionEntry size -> {}", mem::size_of::<TranspositionEntry>());
    }
}