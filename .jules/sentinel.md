## 2025-05-20 - Parser Length Validation
**Vulnerability:** The telemetry parser ignored the `Length` field in the packet header, relying solely on the buffer size. This allowed malformed packets (e.g., claimed length 0 but full payload) to be processed, or packets with trailing garbage to be parsed beyond their boundary.
**Learning:** In binary protocols, always validate explicit length fields against the actual buffer size, and slice the buffer to the declared length before processing. Do not trust that the buffer contains *only* the packet.
**Prevention:** Use `&data[..length]` slicing after validating `length <= data.len()` to enforce the boundary for all downstream parsing logic.

## 2025-05-24 - Integer Truncation in Packet Construction
**Vulnerability:** The manual packet generator used `input.len() as u8` without bounds checking, causing integer truncation when the string length exceeded 255 bytes (e.g., 300 bytes became length 44). This resulted in malformed packets where the internal length field mismatched the actual data payload.
**Learning:** `egui`'s `char_limit` only restricts character count, not byte count, leaving UTF-8 strings vulnerable to exceeding byte-size limits of underlying protocols (e.g., 255 emojis > 255 bytes).
**Prevention:** Always validate and safely truncate dynamic data (e.g., strings) to the protocol's limits before serializing. Use `is_char_boundary` when truncating UTF-8 strings to avoid corrupting multi-byte characters.

## 2025-05-24 - Log Injection in Telemetry Display
**Vulnerability:** The application formatted raw strings from telemetry packets directly into the system log. If a string field (like ) contained newline characters, an attacker could forge fake log entries that appear legitimate to the user.
**Learning:** Even in local GUI applications, untrusted input displayed in logs or lists must be sanitized to prevent misleading output or context confusion.  macro formatting does not automatically escape control characters.
**Prevention:** Always sanitize string fields before logging or displaying them in a line-based format. Use  in Rust to safely display control characters without allocation overhead.

## 2025-05-24 - Log Injection in Telemetry Display
**Vulnerability:** The application formatted raw strings from telemetry packets directly into the system log. If a string field (like `target_id`) contained newline characters, an attacker could forge fake log entries that appear legitimate to the user.
**Learning:** Even in local GUI applications, untrusted input displayed in logs or lists must be sanitized to prevent misleading output or context confusion. `write!` macro formatting does not automatically escape control characters.
**Prevention:** Always sanitize string fields before logging or displaying them in a line-based format. Use `str::escape_debug()` in Rust to safely display control characters without allocation overhead.
