# Overlays

Fixes to the upstream spec, written as [OpenAPI Overlay 1.0](https://spec.openapis.org/overlay/v1.0.0.html)
documents. `spec/openapi.yaml` stays an exact copy of upstream (the weekly sync overwrites it).
These overlays are applied on top of it on every codegen.

- `cargo xtask apply-overlays` (run by `just codegen`) applies every `*.yaml` / `*.yml` file
  in this folder in file-name order, writing `target/codegen/openapi.yaml`, the generator's `inputSpec`.
- Use a numeric prefix to control order, e.g. `01-nullable-dates.yaml`.
- `update` on an object merges recursively (primitives and nested arrays are replaced);
  `update` on an array appends one entry; `remove: true` deletes the selected node.
- A `target` that matches no nodes (or matches a primitive) fails the codegen. This usually means
  upstream changed or fixed that part of the spec. Update or delete the action.

```yaml
overlay: 1.0.0
info:
  title: Booking fixes
  version: 1.0.0
actions:
  - target: $.components.schemas.Booking.properties.endsAt
    description: upstream returns null for open-ended bookings
    update:
      nullable: true
```

For generator-only issues (type or name mappings, normalizer rules) prefer options in
`spec/codegen/rust.yml`. For the shape of the generated code use `spec/templates`.
