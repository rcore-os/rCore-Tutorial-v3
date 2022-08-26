DOCKER_NAME ?= dinghao188/rcore-tutorial
.PHONY: docker build_docker

PWD_CMD := bash -c "pwd"
ifeq ($(OS),Windows_NT)
	PWD_CMD := cmd.exe /c "echo %CD%"
endif

docker:
	docker run --rm -it --mount type=bind,source=$(shell $(PWD_CMD)),destination=/mnt ${DOCKER_NAME}

build_docker: 
	docker build -t ${DOCKER_NAME} .
fmt:
	cd easy-fs; cargo fmt; cd ../easy-fs-fuse cargo fmt; cd ../os ; cargo fmt; cd ../user; cargo fmt; cd ..
