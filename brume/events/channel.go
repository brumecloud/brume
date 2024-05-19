package events

import (
	"errors"
	"slices"
)

type Channel struct {
	id       string
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

func NewChannel(id string) *Channel {
	return &Channel{
		id: id,
	}
}

// publish is instant, error are sent through the error channel
func (c *Channel) Publish(event Event) chan error {
	error_chan := make(chan error)
	go c.process_event(event, error_chan)
	return error_chan
}

func (c *Channel) process_event(event Event, error_chan chan error) {
	all_handlers := c.handlers[event.event_type]

	if all_handlers == nil {
		error_chan <- errors.New("No handler registered for this event")
	}

	slices.SortFunc(all_handlers, func(a, b Handler) int {
		return a.priority - b.priority
	})

	for _, handler := range all_handlers {
		stop_prop, err := handler.f(event.data)
		if err != nil {
			error_chan <- err
		}
		if stop_prop {
			break
		}
	}
}
