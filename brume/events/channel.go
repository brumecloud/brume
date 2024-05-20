package events

import (
	"errors"
	"slices"
)

type Channel struct {
	name     string
	handlers map[string][]Handler
}

type Handler struct {
	f        func(interface{}) (bool, error)
	priority int
}

// event needs to be cast all the time to the right type
type Event struct {
	event_type string
	data       any
}

type EventResult struct {
	error   error
	success interface{}
}

func NewChannel(name string) *Channel {
	return &Channel{
		name: name,
	}
}

// publish is instant, error are sent through the error channel
func (c *Channel) Publish(event Event) chan EventResult {
	error_chan := make(chan EventResult)
	go c.process_event(event, error_chan)
	return error_chan
}

func (c *Channel) process_event(event Event, error_chan chan EventResult) {
	all_handlers := c.handlers[event.event_type]

	if all_handlers == nil {
		res := EventResult{
			error: errors.New("no handler registered for this event"),
		}
		error_chan <- res
	}

	// event have priority and can cancel the propagation
	slices.SortFunc(all_handlers, func(a, b Handler) int {
		return a.priority - b.priority
	})

	for _, handler := range all_handlers {
		stop_prop, err := handler.f(event.data)
		if err != nil {
			res := EventResult{
				error: err,
			}
			error_chan <- res
		}
		if stop_prop {
			break
		}
	}
}
