# Debounced

Utility for building delayed `Future`s and debounced `Stream`s that wait a given
duration before yielding the most recent item.

## Usage
The functionality that is implemented by this crate is pretty simple.  Depending
on whether you want to delay a single future, or debounce an entire stream, you
should either use `delayed` or `debounced`.

### Delaying a Single Value
If you want to delay a single future from resolving to a known value, you can
use `delayed`.

```rust
use debounced::delayed;

let delayed = delayed(42, Duration::from_secs(1)).await;
assert_eq!(start.elapsed().as_secs(), 1);
assert_eq!(delayed, 42);
```

### Debouncing a Stream
If you want to debounce an entire stream, you can use `debounced`.

```rust
use debounced::debounced;

let (mut sender, receiver) = futures_channel::mpsc::channel(1024);
let mut debounced = debounced(receiver, Duration::from_secs(1));
sender.send(21).await;
sender.send(42).await;
assert_eq!(debounced.next().await, Some(42));
assert_eq!(start.elapsed().as_secs(), 1);
```

## Limitations
- __Leading debounce.__ This library currently only implements trailing
  debounce. It does not implement leading debounce.

## License

Copyright 2022 Glacyr B.V.

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
