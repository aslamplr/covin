build-myCovinProxy:
	docker run --rm -v ${PWD}\:/code -v ${HOME}/.cargo/registry\:/root/.cargo/registry -v ${HOME}/.cargo/git\:/root/.cargo/git aslamplr/lambda-rust
	cp ./target/lambda/release/covin-api $(ARTIFACTS_DIR)/bootstrap

