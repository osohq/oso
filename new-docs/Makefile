install:
	$(MAKE) -C .. wasm-build
	$(MAKE) -C ../languages/js build
	npm install -g postcss postcss-cli autoprefixer
	cd themes/tailwind && npm install
	cd themes/oso && npm install && npm run-script build

run:
	hugo server
