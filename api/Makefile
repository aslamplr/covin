
ifeq ($(CI), true)
  $(info CI Environment)
  CMD= echo skipping
else
  $(info Non-CI Environment)
  CMD=docker run --rm -v ${PWD}\:/code -v ${HOME}/.cargo/registry\:/root/.cargo/registry -v ${HOME}/.cargo/git\:/root/.cargo/git aslamplr/lambda-rust
endif


build-myCovinProxy:
	$(CMD)
	cp ./target/lambda/release/covin-api $(ARTIFACTS_DIR)/bootstrap

