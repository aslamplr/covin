
ifeq ($(CI), true)
  $(info CI Environment)
  CMD1= echo skipping
  CMD2= echo skipping
  CMD3= echo skipping
else
  $(info Non-CI Environment)
  CMD1=docker run --rm -e BIN=covin-proxy -v ${PWD}\:/code -v ${HOME}/.cargo/registry\:/root/.cargo/registry -v ${HOME}/.cargo/git\:/root/.cargo/git aslamplr/lambda-rust
  CMD2=docker run --rm -e BIN=covin-alert-engine -v ${PWD}\:/code -v ${HOME}/.cargo/registry\:/root/.cargo/registry -v ${HOME}/.cargo/git\:/root/.cargo/git aslamplr/lambda-rust
  CMD3=docker run --rm -e BIN=covin-api -v ${PWD}\:/code -v ${HOME}/.cargo/registry\:/root/.cargo/registry -v ${HOME}/.cargo/git\:/root/.cargo/git aslamplr/lambda-rust
endif

build-myCovinProxy:
	$(CMD1)
	sleep 10
	$(info docker build myCovinProxy completed)
	cp ./target/lambda/release/covin-proxy $(ARTIFACTS_DIR)/bootstrap

build-myCovinAlertEngine:
	$(CMD2)
	sleep 10
	$(info docker build myCovinAlertEngine completed)
	cp ./target/lambda/release/covin-alert-engine $(ARTIFACTS_DIR)/bootstrap

build-myCovinApi:
	$(CMD3)
	sleep 10
	$(info docker build myCovinApi completed)
	cp ./target/lambda/release/covin-api $(ARTIFACTS_DIR)/bootstrap
