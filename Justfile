docs-install:
  cd ./docs && npm install

docs: docs-install
  cd ./docs && npm run dev
