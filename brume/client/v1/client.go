package client_v1

import (
	"fmt"

	gen "github.com/brume/brume/internal/gen/brume/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

type BrumeClient struct {
	conn *grpc.ClientConn

	authn gen.AuthentificationClient
}

type BrumeClientConfig struct {
	Host string
	Port int
}

func NewClient(cfg BrumeClientConfig) (*BrumeClient, error) {
	address := fmt.Sprint(cfg.Host, ":", cfg.Port)
	var opts []grpc.DialOption
	opts = append(opts, grpc.WithTransportCredentials(insecure.NewCredentials()))

	conn, err := grpc.Dial(address, opts...)

	authn := gen.NewAuthentificationClient(conn)

	client := &BrumeClient{
		conn:  conn,
		authn: authn,
	}

	return client, err
}
