// Code generated by protoc-gen-go. DO NOT EDIT.
// versions:
// 	protoc-gen-go v1.30.0
// 	protoc        (unknown)
// source: brume/v1/organization.proto

package brumev1

import (
	protoreflect "google.golang.org/protobuf/reflect/protoreflect"
	protoimpl "google.golang.org/protobuf/runtime/protoimpl"
	reflect "reflect"
	sync "sync"
)

const (
	// Verify that this generated code is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(20 - protoimpl.MinVersion)
	// Verify that runtime/protoimpl is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(protoimpl.MaxVersion - 20)
)

type Organization struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	Id   string `protobuf:"bytes,1,opt,name=id,proto3" json:"id,omitempty"`
	Name string `protobuf:"bytes,2,opt,name=name,proto3" json:"name,omitempty"`
}

func (x *Organization) Reset() {
	*x = Organization{}
	if protoimpl.UnsafeEnabled {
		mi := &file_brume_v1_organization_proto_msgTypes[0]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *Organization) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*Organization) ProtoMessage() {}

func (x *Organization) ProtoReflect() protoreflect.Message {
	mi := &file_brume_v1_organization_proto_msgTypes[0]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use Organization.ProtoReflect.Descriptor instead.
func (*Organization) Descriptor() ([]byte, []int) {
	return file_brume_v1_organization_proto_rawDescGZIP(), []int{0}
}

func (x *Organization) GetId() string {
	if x != nil {
		return x.Id
	}
	return ""
}

func (x *Organization) GetName() string {
	if x != nil {
		return x.Name
	}
	return ""
}

type GetOrganizationRequest struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	Id string `protobuf:"bytes,1,opt,name=id,proto3" json:"id,omitempty"`
}

func (x *GetOrganizationRequest) Reset() {
	*x = GetOrganizationRequest{}
	if protoimpl.UnsafeEnabled {
		mi := &file_brume_v1_organization_proto_msgTypes[1]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *GetOrganizationRequest) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*GetOrganizationRequest) ProtoMessage() {}

func (x *GetOrganizationRequest) ProtoReflect() protoreflect.Message {
	mi := &file_brume_v1_organization_proto_msgTypes[1]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use GetOrganizationRequest.ProtoReflect.Descriptor instead.
func (*GetOrganizationRequest) Descriptor() ([]byte, []int) {
	return file_brume_v1_organization_proto_rawDescGZIP(), []int{1}
}

func (x *GetOrganizationRequest) GetId() string {
	if x != nil {
		return x.Id
	}
	return ""
}

type CreateOrganizationRequest struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	Name string `protobuf:"bytes,1,opt,name=name,proto3" json:"name,omitempty"`
}

func (x *CreateOrganizationRequest) Reset() {
	*x = CreateOrganizationRequest{}
	if protoimpl.UnsafeEnabled {
		mi := &file_brume_v1_organization_proto_msgTypes[2]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *CreateOrganizationRequest) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*CreateOrganizationRequest) ProtoMessage() {}

func (x *CreateOrganizationRequest) ProtoReflect() protoreflect.Message {
	mi := &file_brume_v1_organization_proto_msgTypes[2]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use CreateOrganizationRequest.ProtoReflect.Descriptor instead.
func (*CreateOrganizationRequest) Descriptor() ([]byte, []int) {
	return file_brume_v1_organization_proto_rawDescGZIP(), []int{2}
}

func (x *CreateOrganizationRequest) GetName() string {
	if x != nil {
		return x.Name
	}
	return ""
}

type ListOrganization struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	Organizations []*Organization `protobuf:"bytes,1,rep,name=organizations,proto3" json:"organizations,omitempty"`
}

func (x *ListOrganization) Reset() {
	*x = ListOrganization{}
	if protoimpl.UnsafeEnabled {
		mi := &file_brume_v1_organization_proto_msgTypes[3]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *ListOrganization) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*ListOrganization) ProtoMessage() {}

func (x *ListOrganization) ProtoReflect() protoreflect.Message {
	mi := &file_brume_v1_organization_proto_msgTypes[3]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use ListOrganization.ProtoReflect.Descriptor instead.
func (*ListOrganization) Descriptor() ([]byte, []int) {
	return file_brume_v1_organization_proto_rawDescGZIP(), []int{3}
}

func (x *ListOrganization) GetOrganizations() []*Organization {
	if x != nil {
		return x.Organizations
	}
	return nil
}

var File_brume_v1_organization_proto protoreflect.FileDescriptor

var file_brume_v1_organization_proto_rawDesc = []byte{
	0x0a, 0x1b, 0x62, 0x72, 0x75, 0x6d, 0x65, 0x2f, 0x76, 0x31, 0x2f, 0x6f, 0x72, 0x67, 0x61, 0x6e,
	0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x12, 0x08, 0x62,
	0x72, 0x75, 0x6d, 0x65, 0x2e, 0x76, 0x31, 0x1a, 0x13, 0x62, 0x72, 0x75, 0x6d, 0x65, 0x2f, 0x76,
	0x31, 0x2f, 0x62, 0x61, 0x73, 0x65, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x22, 0x32, 0x0a, 0x0c,
	0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x12, 0x0e, 0x0a, 0x02,
	0x69, 0x64, 0x18, 0x01, 0x20, 0x01, 0x28, 0x09, 0x52, 0x02, 0x69, 0x64, 0x12, 0x12, 0x0a, 0x04,
	0x6e, 0x61, 0x6d, 0x65, 0x18, 0x02, 0x20, 0x01, 0x28, 0x09, 0x52, 0x04, 0x6e, 0x61, 0x6d, 0x65,
	0x22, 0x28, 0x0a, 0x16, 0x47, 0x65, 0x74, 0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74,
	0x69, 0x6f, 0x6e, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x12, 0x0e, 0x0a, 0x02, 0x69, 0x64,
	0x18, 0x01, 0x20, 0x01, 0x28, 0x09, 0x52, 0x02, 0x69, 0x64, 0x22, 0x2f, 0x0a, 0x19, 0x43, 0x72,
	0x65, 0x61, 0x74, 0x65, 0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e,
	0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x12, 0x12, 0x0a, 0x04, 0x6e, 0x61, 0x6d, 0x65, 0x18,
	0x01, 0x20, 0x01, 0x28, 0x09, 0x52, 0x04, 0x6e, 0x61, 0x6d, 0x65, 0x22, 0x50, 0x0a, 0x10, 0x4c,
	0x69, 0x73, 0x74, 0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x12,
	0x3c, 0x0a, 0x0d, 0x6f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x73,
	0x18, 0x01, 0x20, 0x03, 0x28, 0x0b, 0x32, 0x16, 0x2e, 0x62, 0x72, 0x75, 0x6d, 0x65, 0x2e, 0x76,
	0x31, 0x2e, 0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x52, 0x0d,
	0x6f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x73, 0x32, 0x80, 0x02,
	0x0a, 0x13, 0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x53, 0x65,
	0x72, 0x76, 0x69, 0x63, 0x65, 0x12, 0x4d, 0x0a, 0x0f, 0x47, 0x65, 0x74, 0x4f, 0x72, 0x67, 0x61,
	0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x12, 0x20, 0x2e, 0x62, 0x72, 0x75, 0x6d, 0x65,
	0x2e, 0x76, 0x31, 0x2e, 0x47, 0x65, 0x74, 0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74,
	0x69, 0x6f, 0x6e, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x1a, 0x16, 0x2e, 0x62, 0x72, 0x75,
	0x6d, 0x65, 0x2e, 0x76, 0x31, 0x2e, 0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69,
	0x6f, 0x6e, 0x22, 0x00, 0x12, 0x53, 0x0a, 0x12, 0x43, 0x72, 0x65, 0x61, 0x74, 0x65, 0x4f, 0x72,
	0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x12, 0x23, 0x2e, 0x62, 0x72, 0x75,
	0x6d, 0x65, 0x2e, 0x76, 0x31, 0x2e, 0x43, 0x72, 0x65, 0x61, 0x74, 0x65, 0x4f, 0x72, 0x67, 0x61,
	0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x1a,
	0x16, 0x2e, 0x62, 0x72, 0x75, 0x6d, 0x65, 0x2e, 0x76, 0x31, 0x2e, 0x4f, 0x72, 0x67, 0x61, 0x6e,
	0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x22, 0x00, 0x12, 0x45, 0x0a, 0x14, 0x47, 0x65, 0x74,
	0x55, 0x73, 0x65, 0x72, 0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e,
	0x73, 0x12, 0x0f, 0x2e, 0x62, 0x72, 0x75, 0x6d, 0x65, 0x2e, 0x76, 0x31, 0x2e, 0x45, 0x6d, 0x70,
	0x74, 0x79, 0x1a, 0x1a, 0x2e, 0x62, 0x72, 0x75, 0x6d, 0x65, 0x2e, 0x76, 0x31, 0x2e, 0x4c, 0x69,
	0x73, 0x74, 0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x22, 0x00,
	0x42, 0x98, 0x01, 0x0a, 0x0c, 0x63, 0x6f, 0x6d, 0x2e, 0x62, 0x72, 0x75, 0x6d, 0x65, 0x2e, 0x76,
	0x31, 0x42, 0x11, 0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x50,
	0x72, 0x6f, 0x74, 0x6f, 0x50, 0x01, 0x5a, 0x34, 0x67, 0x69, 0x74, 0x68, 0x75, 0x62, 0x2e, 0x63,
	0x6f, 0x6d, 0x2f, 0x62, 0x72, 0x75, 0x6d, 0x65, 0x2f, 0x62, 0x72, 0x75, 0x6d, 0x65, 0x2f, 0x69,
	0x6e, 0x74, 0x65, 0x72, 0x6e, 0x61, 0x6c, 0x2f, 0x67, 0x65, 0x6e, 0x2f, 0x62, 0x72, 0x75, 0x6d,
	0x65, 0x2f, 0x76, 0x31, 0x3b, 0x62, 0x72, 0x75, 0x6d, 0x65, 0x76, 0x31, 0xa2, 0x02, 0x03, 0x42,
	0x58, 0x58, 0xaa, 0x02, 0x08, 0x42, 0x72, 0x75, 0x6d, 0x65, 0x2e, 0x56, 0x31, 0xca, 0x02, 0x08,
	0x42, 0x72, 0x75, 0x6d, 0x65, 0x5c, 0x56, 0x31, 0xe2, 0x02, 0x14, 0x42, 0x72, 0x75, 0x6d, 0x65,
	0x5c, 0x56, 0x31, 0x5c, 0x47, 0x50, 0x42, 0x4d, 0x65, 0x74, 0x61, 0x64, 0x61, 0x74, 0x61, 0xea,
	0x02, 0x09, 0x42, 0x72, 0x75, 0x6d, 0x65, 0x3a, 0x3a, 0x56, 0x31, 0x62, 0x06, 0x70, 0x72, 0x6f,
	0x74, 0x6f, 0x33,
}

var (
	file_brume_v1_organization_proto_rawDescOnce sync.Once
	file_brume_v1_organization_proto_rawDescData = file_brume_v1_organization_proto_rawDesc
)

func file_brume_v1_organization_proto_rawDescGZIP() []byte {
	file_brume_v1_organization_proto_rawDescOnce.Do(func() {
		file_brume_v1_organization_proto_rawDescData = protoimpl.X.CompressGZIP(file_brume_v1_organization_proto_rawDescData)
	})
	return file_brume_v1_organization_proto_rawDescData
}

var file_brume_v1_organization_proto_msgTypes = make([]protoimpl.MessageInfo, 4)
var file_brume_v1_organization_proto_goTypes = []interface{}{
	(*Organization)(nil),              // 0: brume.v1.Organization
	(*GetOrganizationRequest)(nil),    // 1: brume.v1.GetOrganizationRequest
	(*CreateOrganizationRequest)(nil), // 2: brume.v1.CreateOrganizationRequest
	(*ListOrganization)(nil),          // 3: brume.v1.ListOrganization
	(*Empty)(nil),                     // 4: brume.v1.Empty
}
var file_brume_v1_organization_proto_depIdxs = []int32{
	0, // 0: brume.v1.ListOrganization.organizations:type_name -> brume.v1.Organization
	1, // 1: brume.v1.OrganizationService.GetOrganization:input_type -> brume.v1.GetOrganizationRequest
	2, // 2: brume.v1.OrganizationService.CreateOrganization:input_type -> brume.v1.CreateOrganizationRequest
	4, // 3: brume.v1.OrganizationService.GetUserOrganizations:input_type -> brume.v1.Empty
	0, // 4: brume.v1.OrganizationService.GetOrganization:output_type -> brume.v1.Organization
	0, // 5: brume.v1.OrganizationService.CreateOrganization:output_type -> brume.v1.Organization
	3, // 6: brume.v1.OrganizationService.GetUserOrganizations:output_type -> brume.v1.ListOrganization
	4, // [4:7] is the sub-list for method output_type
	1, // [1:4] is the sub-list for method input_type
	1, // [1:1] is the sub-list for extension type_name
	1, // [1:1] is the sub-list for extension extendee
	0, // [0:1] is the sub-list for field type_name
}

func init() { file_brume_v1_organization_proto_init() }
func file_brume_v1_organization_proto_init() {
	if File_brume_v1_organization_proto != nil {
		return
	}
	file_brume_v1_base_proto_init()
	if !protoimpl.UnsafeEnabled {
		file_brume_v1_organization_proto_msgTypes[0].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*Organization); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
		file_brume_v1_organization_proto_msgTypes[1].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*GetOrganizationRequest); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
		file_brume_v1_organization_proto_msgTypes[2].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*CreateOrganizationRequest); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
		file_brume_v1_organization_proto_msgTypes[3].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*ListOrganization); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
	}
	type x struct{}
	out := protoimpl.TypeBuilder{
		File: protoimpl.DescBuilder{
			GoPackagePath: reflect.TypeOf(x{}).PkgPath(),
			RawDescriptor: file_brume_v1_organization_proto_rawDesc,
			NumEnums:      0,
			NumMessages:   4,
			NumExtensions: 0,
			NumServices:   1,
		},
		GoTypes:           file_brume_v1_organization_proto_goTypes,
		DependencyIndexes: file_brume_v1_organization_proto_depIdxs,
		MessageInfos:      file_brume_v1_organization_proto_msgTypes,
	}.Build()
	File_brume_v1_organization_proto = out.File
	file_brume_v1_organization_proto_rawDesc = nil
	file_brume_v1_organization_proto_goTypes = nil
	file_brume_v1_organization_proto_depIdxs = nil
}
