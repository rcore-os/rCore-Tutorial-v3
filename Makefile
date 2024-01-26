DOCKER_TAG ?= rcore-tutorial-v3:latest
.PHONY: docker build_docker
	
docker:
	docker run --rm -it -v ${PWD}:/mnt -w /mnt --name rcore-tutorial-v3 ${DOCKER_TAG} bash

build_docker: 
	docker build -t ${DOCKER_TAG} --target build .

fmt:
	cd os ; cargo fmt;  cd ..

