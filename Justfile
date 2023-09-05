wasm:
  cd rimu-wasm && wasm-pack build


playground-install:
  cd rimu-playground && npm install

playground: playground-install
  cd rimu-playground && npm run start
