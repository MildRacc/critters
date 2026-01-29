struct Neuron
{
   connections: Box<[NeuralConnection]> 
}

struct NeuralConnection
{
    weight: u8,
    bias: u8
}

struct NeuronLayer
{
    neurons: Box<[Neuron]>
}


pub struct Network
{
    layers: Box<[NeuronLayer]>,

    activationFunc: dyn Fn(i32) -> i32
}

