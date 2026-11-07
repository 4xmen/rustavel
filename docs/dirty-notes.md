
## CSRF in multipart form data:

Multipart token validation: The token must be the first field in the multipart request. If it is not, the request is rejected with a 419 response (fail-closed). This requirement helps reduce the processing overhead of parsing multipart requests.

The peek_multipart function reads body chunks only until the first multipart field is complete. It then restores the request body by combining the bytes already read with the remaining stream. Only the initial few kilobytes are buffered, so large uploads continue to stream without being fully loaded into memory.

[csrf.rs](../core/src/http/csrf.rs)
---------


