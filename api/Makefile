
ifeq ($(CI), true)
  $(info CI Environment)
  CMD= echo skipping
  CMD2= echo skipping
else
  $(info Non-CI Environment)
  CMD=docker run --rm -v ${PWD}\:/code -v ${HOME}/.cargo/registry\:/root/.cargo/registry -v ${HOME}/.cargo/git\:/root/.cargo/git aslamplr/lambda-rust
  CMD2=docker run --rm -e BIN=alert-engine -v ${PWD}\:/code -v ${HOME}/.cargo/registry\:/root/.cargo/registry -v ${HOME}/.cargo/git\:/root/.cargo/git aslamplr/lambda-rust
endif


build-myCovinProxy:
	$(CMD)
	cp ./target/lambda/release/covin-api $(ARTIFACTS_DIR)/bootstrap

build-myCovinAlertEngine:
	$(CMD2)
	cp ./target/lambda/release/alert-engine $(ARTIFACTS_DIR)/bootstrap
