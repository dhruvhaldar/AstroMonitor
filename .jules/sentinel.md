## 2025-05-20 - Parser Length Validation
**Vulnerability:** The telemetry parser ignored the `Length` field in the packet header, relying solely on the buffer size. This allowed malformed packets (e.g., claimed length 0 but full payload) to be processed, or packets with trailing garbage to be parsed beyond their boundary.
**Learning:** In binary protocols, always validate explicit length fields against the actual buffer size, and slice the buffer to the declared length before processing. Do not trust that the buffer contains *only* the packet.
**Prevention:** Use `&data[..length]` slicing after validating `length <= data.len()` to enforce the boundary for all downstream parsing logic.

## 2025-05-24 - Integer Truncation in Packet Construction
**Vulnerability:** The manual packet generator used `input.len() as u8` without bounds checking, causing integer truncation when the string length exceeded 255 bytes (e.g., 300 bytes became length 44). This resulted in malformed packets where the internal length field mismatched the actual data payload.
**Learning:** `egui`'s `char_limit` only restricts character count, not byte count, leaving UTF-8 strings vulnerable to exceeding byte-size limits of underlying protocols (e.g., 255 emojis > 255 bytes).
**Prevention:** Always validate and safely truncate dynamic data (e.g., strings) to the protocol's limits before serializing. Use `is_char_boundary` when truncating UTF-8 strings to avoid corrupting multi-byte characters.

## 2025-05-24 - Log Injection in Telemetry Display
**Vulnerability:** The application formatted raw strings from telemetry packets directly into the system log. If a string field (like `target_id`) contained newline characters, an attacker could forge fake log entries that appear legitimate to the user.
**Learning:** Even in local GUI applications, untrusted input displayed in logs or lists must be sanitized to prevent misleading output or context confusion. `write!` macro formatting does not automatically escape control characters.
**Prevention:** Always sanitize string fields before logging or displaying them in a line-based format. Use `str::escape_debug()` in Rust to safely display control characters without allocation overhead.

## 2025-05-24 - CSV Injection in Log Export
**Vulnerability:** The `StarTracker` `target_id` field was displayed directly in logs without escaping special characters that trigger formulas in spreadsheet software (like Excel/LibreOffice). If a user copied logs containing a malicious ID (e.g., `=cmd|...`) and pasted them into a spreadsheet, it could result in arbitrary code execution or data exfiltration.
**Learning:** "Copy to Clipboard" functionality in data-heavy applications is an often-overlooked attack vector. Sanitizing for display (HTML/Console) is not enough; one must also consider the destination of the data (Spreadsheets/CSV). Furthermore, spreadsheets often ignore leading whitespace before a formula character, allowing bypasses if only the very first character is checked.
**Prevention:** When formatting user input that might be exported to CSV or pasted into spreadsheets, check if the string starts with `=`, `+`, `-`, or `@` after trimming leading whitespace (`trim_start()`), and prepend a single quote `'` to force it to be treated as text.

## 2025-05-24 - Invalid Sensor Data (NaN) Ignored
**Vulnerability:** The telemetry monitor relied on `<` and `>` comparisons for thresholds (e.g. `battery < 20.0`). Because `NaN < x` is always false, sensors reporting `NaN` (due to failure or attack) bypassed critical alerts, potentially masking system failures.
**Learning:** Floating-point comparisons are not sufficient for safety-critical monitoring. `NaN` and `Infinity` are valid float values but invalid sensor readings that must be explicitly handled.
**Prevention:** Use `.is_finite()` to validate all sensor inputs before applying threshold logic. Treat non-finite values as immediate sensor failures.

## 2025-05-24 - Unchecked Control Characters in Identifiers
**Vulnerability:** The `StarTracker` `target_id` field was accepted as any valid UTF-8 string, including control characters like `\n` or `\0`. This could allow log injection at the source or confuse downstream systems, even if display layers escape it.
**Learning:** Business logic often assumes identifiers are "names" without enforcing character set constraints. "Valid UTF-8" is not the same as "Valid Identifier".
**Prevention:** Enforce strict character set validation (e.g., printable characters only) at the ingestion/monitoring layer for all identifier fields.

## 2024-03-24 - Sensor Failure Misclassification
**Vulnerability:** Invalid sensor readings (e.g., negative confidence, >100% battery) were treated as "Low/High" values triggering lower-severity alerts (Info/Warning) instead of Critical Sensor Failures.
**Learning:** Threshold-based logic (`value < threshold`) implicitly assumes valid input ranges. Negative values satisfy `<` checks but represent fundamental invalidity.
**Prevention:** Enforce strict domain validation (e.g. 0.0-1.0 range) *before* applying business logic thresholds. Treat out-of-domain values as system failures, not process deviations.

## 2025-05-27 - Unchecked Telemetry Temperatures (Thermal)
**Vulnerability:** The `Monitor` logic checked that temperature values were `finite` but did not validate the physical limit of absolute zero (-273.15 °C). This allowed nonsensical temperatures (e.g., -300.0 °C) to be processed.
**Learning:** Float validation alone is insufficient for sensor data. The physical constraints of the real-world values being represented (e.g., absolute zero) must be validated to prevent logic errors and misclassification of failures.
**Prevention:** Implement strict domain validation for all physical quantities, ensuring values fall within physically possible limits before business logic processes them.

## 2025-05-27 - Unchecked Telemetry Coordinates (RA/Dec)
**Vulnerability:** The `Monitor` logic checked that coordinate values were `finite` but did not validate their physical ranges (RA: 0-360, Dec: -90-+90). This allowed nonsensical values (e.g., RA=400.0) to be processed without raising a `SensorFailure`.
**Learning:** Physical systems often have implicit constraints that are not enforced by data types (e.g., `f64` covers all real numbers). "Valid float" != "Valid coordinate".
**Prevention:** Implement strict domain validation for all physical quantities at the ingestion layer, ensuring values fall within their defined physical limits before any business logic is applied.

## 2025-05-27 - Monitor Configuration Bypass (NaN Thresholds)
**Vulnerability:** The `Monitor` struct exposed public fields for thresholds (e.g., `min_battery_level`). If set to `NaN` (accidentally or maliciously), the alert logic `value < NaN` always evaluates to false, silently disabling critical alerts.
**Learning:** Publicly mutable configuration structs bypass invariants. Type safety alone (`f64`) is insufficient to guarantee valid configuration state.
**Prevention:** Encapsulate configuration fields as private members. Use constructor and setter methods that enforce validation logic (e.g., reject `NaN` / `Inf`) to ensure the system is always in a valid, safe state.

## 2025-05-24 - CSV Injection Bypass via Control Characters
**Vulnerability:** The CSV injection sanitization logic used `trim_start()` before checking for formula characters (`=`, `+`, `-`, `@`). However, `trim_start()` only removes Unicode whitespace, allowing attackers to bypass the filter by prepending invisible control characters (like `\x08` Backspace) or zero-width spaces (`\u{200B}`). Spreadsheets often ignore these characters when executing formulas.
**Learning:** When sanitizing input for CSV export, removing only standard whitespace is insufficient to prevent formula execution bypasses. Invisible characters and control characters must also be trimmed before checking for formula triggers.
**Prevention:** Use `trim_start_matches(|c: char| c.is_whitespace() || c.is_control() || c == '\u{200B}' || c == '\u{FEFF}')` to strip all invisible or control characters before verifying if the string starts with a formula character.

## 2025-05-27 - Unchecked Telemetry Power Readings (Voltage/Current)
**Vulnerability:** The `Monitor` logic checked that voltage and current values were `finite` but did not validate their physical ranges (voltage: >=0, current: >=0) for typical space telemetry power buses. This allowed nonsensical negative values to be processed without raising a `SensorFailure`.
**Learning:** Physical systems often have implicit constraints that are not enforced by data types. Negative voltage and current on systems that don't support them represent sensor failures or spoofed data.
**Prevention:** Implement strict domain validation for all physical quantities at the ingestion layer, ensuring values fall within their defined physical limits before any business logic is applied.

## 2025-05-28 - CSV Injection Bypass via Tab and Carriage Return
**Vulnerability:** The CSV injection sanitization logic checked if the payload started with `=`, `+`, `-`, or `@`. However, it missed `\t` (Tab) and `\r` (Carriage Return), which can also act as triggers or modifiers for formula injection in some spreadsheet programs (like Excel/LibreOffice), especially when leading whitespace is bypassed or ignored.
**Learning:** Checking for standard mathematical prefixes is insufficient for comprehensive CSV injection protection. Control characters and formatting characters like tabs and carriage returns must also be explicitly included in the prefix validation check.
**Prevention:** Extend the prefix validation list to explicitly include `\t` and `\r` (e.g., `c == '=' || c == '+' || c == '-' || c == '@' || c == '\t' || c == '\r'`) after trimming irrelevant whitespace.

## 2025-05-28 - CSV Injection Bypass via Delimiters
**Vulnerability:** The CSV injection sanitization logic checked if the payload started with standard formula or control characters. However, if a payload started with common CSV delimiters like `,`, `;`, or `|` (e.g., `,=cmd|...`), the sanitization failed. When this text is exported to CSV and parsed by a spreadsheet application, the delimiter forces the malicious formula into the adjacent cell, executing it.
**Learning:** CSV injection is not just about the absolute first character of the string; it's about the first character of the resulting *cell* after CSV delimiters are parsed. Prepending a delimiter to a formula is a common bypass if the sanitization logic does not also quote or sanitize delimiters.
**Prevention:** Include common CSV delimiters (`,`, `;`, `|`) in the prefix validation check. Prepending a quote to any string that begins with a delimiter ensures it remains a harmless string within a single cell, rather than triggering formula execution in the next cell.

## 2025-05-28 - CSV Injection Bypass via Quotes
**Vulnerability:** The CSV injection sanitization logic correctly checked for various whitespace, control characters, formula characters, and delimiters. However, it failed to account for leading quotation marks (`"` and `'`). If an attacker prepends a formula with quotation marks (e.g., `"=cmd|..."`), certain spreadsheet applications will automatically strip the leading quotes when parsing the CSV or upon pasting the data, successfully executing the hidden formula underneath.
**Learning:** Relying purely on the first non-whitespace/control character is insufficient for mitigating CSV formula injections, as popular spreadsheet parsers perform multiple preprocessing steps (like quote removal) before evaluating cells as formulas.
**Prevention:** When skipping over characters prior to checking for malicious formula prefixes, include quotation marks (`"` and `'`) in the list of ignored prefix characters, ensuring the sanitizer detects formulas nested directly inside quotes.
## 2025-05-29 - CSV Injection Bypass via Hangul Filler
**Vulnerability:** The CSV injection sanitization logic checked if the payload started with standard formula or control characters. However, it failed to account for the Hangul Filler character (`\u{3164}`). This character is not considered a standard Unicode whitespace or control character, so it wasn't stripped. When prepended to a malicious formula, it bypassed the prefix check, but spreadsheet applications often parse it as a spacer and execute the subsequent formula.
**Learning:** Checking for standard control and whitespace characters is insufficient when there are obscure characters (like `\u{3164}`) that act as invisible spacers but bypass standard categorizations (`is_whitespace()`, `is_control()`). Attackers can leverage these specific characters to evade standard sanitization checks.
**Prevention:** In addition to checking standard whitespace and control characters, explicitly maintain a blocklist of known "filler" or bypass characters (such as `\u{3164}`) to properly sanitize strings prior to checking for CSV formula prefixes.
