// Code generated by protoc-gen-go-grpc. DO NOT EDIT.
// versions:
// - protoc-gen-go-grpc v1.3.0
// - protoc             (unknown)
// source: brume/v1/user.proto

package brumev1

import (
	context "context"
	grpc "google.golang.org/grpc"
	codes "google.golang.org/grpc/codes"
	status "google.golang.org/grpc/status"
)

// This is a compile-time assertion to ensure that this generated file
// is compatible with the grpc package it is being compiled against.
// Requires gRPC-Go v1.32.0 or later.
const _ = grpc.SupportPackageIsVersion7

const (
	UserService_GetMe_FullMethodName = "/brume.v1.UserService/GetMe"
)

// UserServiceClient is the client API for UserService service.
//
// For semantics around ctx use and closing/ending streaming RPCs, please refer to https://pkg.go.dev/google.golang.org/grpc/?tab=doc#ClientConn.NewStream.
type UserServiceClient interface {
	GetMe(ctx context.Context, in *Empty, opts ...grpc.CallOption) (*User, error)
}

type userServiceClient struct {
	cc grpc.ClientConnInterface
}

func NewUserServiceClient(cc grpc.ClientConnInterface) UserServiceClient {
	return &userServiceClient{cc}
}

func (c *userServiceClient) GetMe(ctx context.Context, in *Empty, opts ...grpc.CallOption) (*User, error) {
	out := new(User)
	err := c.cc.Invoke(ctx, UserService_GetMe_FullMethodName, in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

// UserServiceServer is the server API for UserService service.
// All implementations must embed UnimplementedUserServiceServer
// for forward compatibility
type UserServiceServer interface {
	GetMe(context.Context, *Empty) (*User, error)
	mustEmbedUnimplementedUserServiceServer()
}

// UnimplementedUserServiceServer must be embedded to have forward compatible implementations.
type UnimplementedUserServiceServer struct {
}

func (UnimplementedUserServiceServer) GetMe(context.Context, *Empty) (*User, error) {
	return nil, status.Errorf(codes.Unimplemented, "method GetMe not implemented")
}
func (UnimplementedUserServiceServer) mustEmbedUnimplementedUserServiceServer() {}

// UnsafeUserServiceServer may be embedded to opt out of forward compatibility for this service.
// Use of this interface is not recommended, as added methods to UserServiceServer will
// result in compilation errors.
type UnsafeUserServiceServer interface {
	mustEmbedUnimplementedUserServiceServer()
}

func RegisterUserServiceServer(s grpc.ServiceRegistrar, srv UserServiceServer) {
	s.RegisterService(&UserService_ServiceDesc, srv)
}

func _UserService_GetMe_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(Empty)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(UserServiceServer).GetMe(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: UserService_GetMe_FullMethodName,
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(UserServiceServer).GetMe(ctx, req.(*Empty))
	}
	return interceptor(ctx, in, info, handler)
}

// UserService_ServiceDesc is the grpc.ServiceDesc for UserService service.
// It's only intended for direct use with grpc.RegisterService,
// and not to be introspected or modified (even as a copy)
var UserService_ServiceDesc = grpc.ServiceDesc{
	ServiceName: "brume.v1.UserService",
	HandlerType: (*UserServiceServer)(nil),
	Methods: []grpc.MethodDesc{
		{
			MethodName: "GetMe",
			Handler:    _UserService_GetMe_Handler,
		},
	},
	Streams:  []grpc.StreamDesc{},
	Metadata: "brume/v1/user.proto",
}

const (
	Authentification_PasswordLogin_FullMethodName = "/brume.v1.Authentification/PasswordLogin"
)

// AuthentificationClient is the client API for Authentification service.
//
// For semantics around ctx use and closing/ending streaming RPCs, please refer to https://pkg.go.dev/google.golang.org/grpc/?tab=doc#ClientConn.NewStream.
type AuthentificationClient interface {
	PasswordLogin(ctx context.Context, in *LoginRequest, opts ...grpc.CallOption) (*LoginResponse, error)
}

type authentificationClient struct {
	cc grpc.ClientConnInterface
}

func NewAuthentificationClient(cc grpc.ClientConnInterface) AuthentificationClient {
	return &authentificationClient{cc}
}

func (c *authentificationClient) PasswordLogin(ctx context.Context, in *LoginRequest, opts ...grpc.CallOption) (*LoginResponse, error) {
	out := new(LoginResponse)
	err := c.cc.Invoke(ctx, Authentification_PasswordLogin_FullMethodName, in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

// AuthentificationServer is the server API for Authentification service.
// All implementations must embed UnimplementedAuthentificationServer
// for forward compatibility
type AuthentificationServer interface {
	PasswordLogin(context.Context, *LoginRequest) (*LoginResponse, error)
	mustEmbedUnimplementedAuthentificationServer()
}

// UnimplementedAuthentificationServer must be embedded to have forward compatible implementations.
type UnimplementedAuthentificationServer struct {
}

func (UnimplementedAuthentificationServer) PasswordLogin(context.Context, *LoginRequest) (*LoginResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method PasswordLogin not implemented")
}
func (UnimplementedAuthentificationServer) mustEmbedUnimplementedAuthentificationServer() {}

// UnsafeAuthentificationServer may be embedded to opt out of forward compatibility for this service.
// Use of this interface is not recommended, as added methods to AuthentificationServer will
// result in compilation errors.
type UnsafeAuthentificationServer interface {
	mustEmbedUnimplementedAuthentificationServer()
}

func RegisterAuthentificationServer(s grpc.ServiceRegistrar, srv AuthentificationServer) {
	s.RegisterService(&Authentification_ServiceDesc, srv)
}

func _Authentification_PasswordLogin_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(LoginRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(AuthentificationServer).PasswordLogin(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: Authentification_PasswordLogin_FullMethodName,
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(AuthentificationServer).PasswordLogin(ctx, req.(*LoginRequest))
	}
	return interceptor(ctx, in, info, handler)
}

// Authentification_ServiceDesc is the grpc.ServiceDesc for Authentification service.
// It's only intended for direct use with grpc.RegisterService,
// and not to be introspected or modified (even as a copy)
var Authentification_ServiceDesc = grpc.ServiceDesc{
	ServiceName: "brume.v1.Authentification",
	HandlerType: (*AuthentificationServer)(nil),
	Methods: []grpc.MethodDesc{
		{
			MethodName: "PasswordLogin",
			Handler:    _Authentification_PasswordLogin_Handler,
		},
	},
	Streams:  []grpc.StreamDesc{},
	Metadata: "brume/v1/user.proto",
}
