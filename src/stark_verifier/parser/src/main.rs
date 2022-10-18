use std::fs::File;
use std::io::{ BufReader, Read, Result };
use serde::{ Deserialize, Serialize };
use winterfell::StarkProof;

use std::env;

use winter_air::proof::{Context};
use winter_air::TraceLayout;

mod memory;
use memory::{ MemoryEntry, Writeable, DynamicMemory };

#[derive(Serialize, Deserialize)]
struct ProofData {
    input_bytes: Vec<u8>,
    proof_bytes: Vec<u8>,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let proof_path = &args[1];

    let file = File::open(proof_path)?;
    let mut buf_reader = BufReader::new(file);
    
    let mut data = Vec::new();
    buf_reader.read_to_end(&mut data).expect("Unable to read data");
    
    let data: ProofData = bincode::deserialize( &data ).unwrap();
    let proof = StarkProof::from_bytes(&data.proof_bytes).unwrap();

    let mut memories = Vec::<Vec<MemoryEntry>>::new();
    let mut dynamic_memory = DynamicMemory::new(&mut memories);

    proof.write_into(&mut dynamic_memory);

    let memory = dynamic_memory.serialize();

    let json_arr = serde_json::to_string(&memory)?;
    println!( "{}", json_arr );

    Ok(())
}


impl Writeable for u8 {
    fn write_into(&self, target: &mut DynamicMemory) {
        target.write_value(*self as u64)
    }
}

impl Writeable for u64 {
    fn write_into(&self, target: &mut DynamicMemory) {
        target.write_value(*self)
    }
}

impl Writeable for usize {
    fn write_into(&self, target: &mut DynamicMemory) {
        target.write_value(*self as u64)
    }
}


impl Writeable for TraceLayout {

    fn write_into(&self, target: &mut DynamicMemory) {
        let mut aux_segement_widths = Vec::new();
        let mut aux_segment_rands = Vec::new();

        for i in 0..self.num_aux_segments() {
            aux_segement_widths.push( self.get_aux_segment_width(i) );
            aux_segment_rands.push( self.get_aux_segment_rand_elements(i) );
        }

        self.main_trace_width().write_into(target);
        self.num_aux_segments().write_into(target);
        target.write_array(aux_segement_widths);
        target.write_array(aux_segment_rands);
    }

}

impl Writeable for Context {

    fn write_into(&self, target: &mut DynamicMemory) {
        
        self.trace_layout().write_into(target);

        self.trace_length().write_into(target); // Do not serialize as a power of two
        
        self.get_trace_info().meta().len().write_into(target);
        target.write_array(self.get_trace_info().meta().to_vec());
        
        self.field_modulus_bytes().len().write_into(target);
        target.write_array(self.field_modulus_bytes().to_vec());
    }

}


impl Writeable for StarkProof {
    
    fn write_into(&self, target: &mut DynamicMemory){
        self.context.write_into(target);
        self.pow_nonce.write_into(target);
        
    }
    
}
