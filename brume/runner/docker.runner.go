package runner

type DockerExecutor struct{}

func (d *DockerExecutor) Init() {}

func (d *DockerExecutor) Run() {}

func (d *DockerExecutor) Kill() {}

func (d *DockerExecutor) Logs() {}

func (d *DockerExecutor) Metrics() {}

func (d *DockerExecutor) Check() (bool, error) {
	return true, nil
}

func (d *DockerExecutor) CheckRunner() (bool, error) {
	return true, nil
}
