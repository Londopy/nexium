# Project: a neural network

No library, no framework, no external data: a two-layer network trained by
backpropagation, in a hundred and seventy lines of the language you have
now learned all of. It solves XOR, which a single layer cannot, and then a
harder problem, telling two rings of points apart, on data it has never
seen.

{{include topo/code/nn.nx}}

{{output topo/code/nn.expected}}

It trains in well under a second in debug mode, and `nx run
topo/code/nn.nx --mode fast` is faster still; `random.seed(7)` makes the run
reproducible, so the book can show its numbers.

## The pieces

**A matrix** is a struct with a `List(f64)` and two methods, `at` and
`set`, that turn row and column into an index. `Matrix.random` fills it
with small values in `[-scale, scale]`; that is the whole of the linear
algebra, because a network this size needs no more.

**A layer** holds its weights and biases and, after each forward pass, what
it saw and what it produced. `forward` computes `sigmoid(W x + b)` one
neuron at a time. `backward` receives the gradient of the loss with respect
to its output, folds in the sigmoid's derivative (`y * (1 - y)`), updates
every weight by `lr * delta * input`, and returns the gradient with respect
to its input for the layer below. That function is backpropagation; there
is nothing else to it.

**A network** is a list of layers. `predict` runs them forward; `train_one`
runs one example forward, measures the squared error, and runs the layers
backward in reverse order, handing each the gradient the one above computed.
The loop in `train` does that for every example, some hundreds of times.

## Where the language shows

```nexium
    fn forward(self: *mut Self, x: []f64) -> List(f64) {
        self.input = x.to_owned()
```

Ownership decides what gets copied. `x` is a slice, a view of the caller's
data; the layer needs to keep the input for `backward`, so it takes a copy
with `to_owned`, and that is the only copy in the forward pass. `predict`
threads a `List(f64)` through the layers, moving it into each `forward`
call (`cur = self.layers[i].forward(cur[..])`); nothing is retained, nothing
is garbage collected, and `nx leaks topo/code/nn.nx` reports that all
seven hundred thousand allocations the run makes were released.

```nexium
        var i = self.layers.len
        while i > 0 {
            i -= 1
            grad = self.layers[i].backward(grad[..], lr)
        }
```

Indexing `self.layers[i]` on a `*mut Self` and calling a `*mut Self` method
on the element mutates the layer in place. The checker knows `i` is in
range here from the loop's shape; the arithmetic on `f64` cannot panic; the
compiled loop is what you would have written in C.

```nexium
fn xor_data() -> Data {
    ...
            d.xs.append(List(f64).from([a as f64, b as f64]))
```

A `Data` holds two `List(List(f64))`, rows of inputs and rows of targets;
`List(f64).from([...])` builds a row from an array literal. Nested owning
containers own their contents all the way down and are freed all the way
down, with no code written for it.

## What to try

- **Momentum or a learning-rate schedule** in `backward`: a few lines each,
  and both make the rings converge in fewer epochs.
- **`for parallel` over the held-out set** in `accuracy`: prediction does
  not modify the network, so with a `*Network` and results written to a
  slice by index it parallelizes as chapter 16 shows. (Training does modify
  it, and the compiler will not let a `for parallel` body do that; that is
  the effect system telling you why it needs a different algorithm.)
- **A third layer.** `Network.new([2, 8, 4, 1][..])` is the only change.
- **Real data.** `@embedFile` (chapter 13) a CSV of measurements, parse it
  with `split(",")` and `parse_float`, and the same code trains on it.
- **Ship it.** `export(c)` a `predict` that takes and returns `[]f64`
  buffers, and call the trained network from Python.

Next: [the tools](22-tools.html).
