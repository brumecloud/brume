package resolver

import service "brume.dev/service/model"

type ServiceResolver struct {
	s *service.Service
	q *QueryResolver
}

func (s *ServiceResolver) Name() string {
	return s.s.Name
}

func (s *ServiceResolver) Id() string {
	return s.s.ID.String()
}
