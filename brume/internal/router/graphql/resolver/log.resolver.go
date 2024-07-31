package resolver

import log_model "brume.dev/logs/model"

type LogResolver struct {
	l *log_model.Log
}

func (l *LogResolver) Message() string {
	return l.l.Message
}

func (l *LogResolver) Level() string {
	return l.l.Level
}
