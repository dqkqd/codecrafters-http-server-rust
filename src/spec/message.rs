#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct FieldName(pub Vec<u8>);

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct FieldContent(pub Vec<u8>);

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct FieldValue(pub Vec<FieldContent>);

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct MessageHeader {
    pub field_name: FieldName,
    pub field_value: Option<FieldValue>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct MessageBody(pub Vec<u8>);
