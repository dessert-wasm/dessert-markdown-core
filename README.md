Dessert Markdown
============
This library is the base API to transform a markdown file to html file, written in Rust for WebAssembly.

## Summary
* [Installation](#installation)
* [API](#api)
* [Building](#building)

## Installation

> Note:  
Although this module is not supposed to be used by itself, it can still be used as a standalone module.  
dessert-markdown-core is depended on by [dessert-showdown](https://github.com/dessert-wasm/dessert-showdown)
```sh
npm install dessert-markdown-core
```

## API
Here is a quick lookup of how you can use dessert-markdown 

```javascript
var showdown  = require('dessert-markdown-core'),
    converter = new showdown.Converter(),
    text      = '# hello, markdown!',
    html      = converter.makeHtml(text);
```

### Output 
This example should output...
```html
    <h1 id="hellomarkdown">hello, markdown!</h1>
```
## Building
The project is built using [wasm-pack](https://github.com/rustwasm/wasm-pack)
To build the project, run
```sh
wasm-pack build
```