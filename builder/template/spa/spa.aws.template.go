package spa

type SpaAwsTemplate struct {
	SpaTemplate
}

func (t *SpaAwsTemplate) Preview() error {
	return nil
}

func (t *SpaAwsTemplate) Up() error {
	return nil
}

func (t *SpaAwsTemplate) Destroy() error {
	return nil
}
