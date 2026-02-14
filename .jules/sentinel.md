## 2025-05-20 - Parser Length Validation
**Vulnerability:** The telemetry parser ignored the `Length` field in the packet header, relying solely on the buffer size. This allowed malformed packets (e.g., claimed length 0 but full payload) to be processed, or packets with trailing garbage to be parsed beyond their boundary.
**Learning:** In binary protocols, always validate explicit length fields against the actual buffer size, and slice the buffer to the declared length before processing. Do not trust that the buffer contains *only* the packet.
**Prevention:** Use `&data[..length]` slicing after validating `length <= data.len()` to enforce the boundary for all downstream parsing logic.
