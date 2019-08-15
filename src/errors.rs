error_chain!{
    errors {
        SerializeError(e: quick_protobuf::Error) {
            description("Serialization error")
            display("Serialization error as '{:?}'", e)
        }
    }
}