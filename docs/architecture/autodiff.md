# Automatic differentiation

Bedrock treats automatic differentiation (autodiff) as a compiler
transformation over typed tensor programs. It does not attach an ambient tape
or mutable gradient field to every tensor.

> **Status:** This is a future milestone after fixed-rank tensors. The source
> syntax is illustrative.

## Contract

An autodiff request is explicit:

```bedrock
loss, grads = value and grad train_step, model, batch
```

The compiler returns the primal value and an owned gradient value. Parameters
do not acquire hidden `.grad` state. Accumulation, clipping, distribution, and
optimizer updates consume or produce explicit values.

Backend, placement, and autodiff capability do not appear as tensor generic
parameters. Tensor types describe elements and shapes. High-level
intermediate representation (HIR) records placement and transformation state.

## Differentiable code

Every differentiated operation needs a derivative rule. The compiler rejects
an operation without one. A custom graphics processing unit (GPU) `kn` must
supply a derivative or lower from tensor operations whose derivatives are
known.

External effects are not differentiable. Input/output, global mutation, and
host callbacks are rejected inside a differentiated region. Local mutation is
legal when the compiler can convert it to value flow without changing visible
behavior.

The first milestone supports reverse mode for scalar losses. The intermediate
representation must still model vector-Jacobian products (VJPs) and
Jacobian-vector products (JVPs) so higher-order differentiation can compose
later.

## Compilation

The compiler performs these steps:

1. Activity analysis removes values that cannot affect the selected result.
2. HIR differentiation produces forward and backward functions.
3. Multi-Level Intermediate Representation (MLIR) optimizes both functions.
4. A cost model chooses whether to save or recompute pure intermediate values.
5. Ownership and liveness assign, reuse, and release gradient buffers.
6. Central processing unit (CPU) and GPU lowering emit the selected target code.

Recomputation never repeats an effectful operation. Fusion and checkpointing
share one cost model because either decision changes the value of the other.

Distributed lowering may start a gradient reduction when that gradient becomes
ready. It does not wait for the complete backward pass when dependencies allow
communication and computation to overlap.

## Dynamic execution

Compiled differentiation is the default. A future eager mode may build a
runtime graph for explicitly dynamic research code. The mode must be selected
explicitly and cannot act as a fallback when compiled differentiation fails.

Both modes must return the same gradient structure and satisfy the same
derivative rules.

## Required tests

- Compare every primitive derivative with a numerical reference.
- Exclude inactive values from the backward graph.
- Preserve branch and loop semantics in forward and reverse passes.
- Prove gradient values carry no hidden parameter mutation.
- Release temporary gradients after their final use.
- Prove checkpointed and stored executions agree.
- Reject recomputation across an effect.
- Match CPU and GPU gradients within a declared numerical tolerance.

## References

- [JAX transformations](https://docs.jax.dev/en/latest/key-concepts.html)
- [JAX stateful computations](https://docs.jax.dev/en/latest/stateful-computations.html)
- [Burn explicit gradients](https://burn.dev/books/burn/building-blocks/autodiff.html)
- [Burn backend-generic removal](https://github.com/tracel-ai/burn/pull/4717)
- [Burn gradient storage](https://github.com/tracel-ai/burn/blob/b6e27bdca620fbbc15e524c7088a7711f1a999f1/crates/burn-autodiff/src/grads.rs)
