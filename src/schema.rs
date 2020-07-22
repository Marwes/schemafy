pub type PositiveInteger = i64 ; pub type PositiveIntegerDefault0 = serde_json
:: Value ; pub type SchemaArray = Vec < Schema > ; #
[serde(rename = "simpleTypes")] #
[derive(Clone, PartialEq, Debug, Deserialize, Serialize)] pub enum SimpleTypes
{
    # [serde(rename = "array")] Array, # [serde(rename = "boolean")] Boolean,
    # [serde(rename = "integer")] Integer, # [serde(rename = "null")] Null, #
    [serde(rename = "number")] Number, # [serde(rename = "object")] Object, #
    [serde(rename = "string")] String
} pub type StringArray = Vec < String > ; #
[derive(Clone, PartialEq, Debug, Deserialize, Serialize)] pub struct Schema
{
    # [serde(rename = "$ref")] pub _ref : Option < String >, #
    [serde(rename = "$schema")] pub _schema : Option < String >, #
    [serde(rename = "0")] pub _0 : Option < Box < Schema >>, #
    [serde(rename = "additionalItems")] pub additional_items : Option <
    serde_json :: Value >, # [serde(rename = "additionalProperties")] pub
    additional_properties : Option < serde_json :: Value >, #
    [serde(rename = "allOf")] pub all_of : Option < SchemaArray >, #
    [serde(rename = "anyOf")] pub any_of : Option < SchemaArray >, pub default
    : Option < serde_json :: Value >, # [serde(default)] pub definitions : ::
    std :: collections :: BTreeMap < String, Schema >, pub dependencies :
    Option < :: std :: collections :: BTreeMap < String, serde_json :: Value
    >>, pub description : Option < String >, # [serde(rename = "enum")] pub
    enum_ : Option < Vec < serde_json :: Value >>, #
    [serde(rename = "enumNames")] pub enum_names : Option < Vec < serde_json
    :: Value >>, # [serde(rename = "exclusiveMaximum")] pub exclusive_maximum
    : Option < bool >, # [serde(rename = "exclusiveMinimum")] pub
    exclusive_minimum : Option < bool >, pub id : Option < String >, #
    [serde(default)] # [serde(with = "::schemafy_core::one_or_many")] pub
    items : Vec < Schema >, # [serde(rename = "maxItems")] pub max_items :
    Option < PositiveInteger >, # [serde(rename = "maxLength")] pub max_length
    : Option < PositiveInteger >, # [serde(rename = "maxProperties")] pub
    max_properties : Option < PositiveInteger >, pub maximum : Option < f64 >,
    # [serde(rename = "minItems")] pub min_items : Option <
    PositiveIntegerDefault0 >, # [serde(rename = "minLength")] pub min_length
    : Option < PositiveIntegerDefault0 >, # [serde(rename = "minProperties")]
    pub min_properties : Option < PositiveIntegerDefault0 >, pub minimum :
    Option < f64 >, # [serde(rename = "multipleOf")] pub multiple_of : Option
    < f64 >, pub not : Option < Box < Schema >>, #
    [serde(rename = "numericalEnum")] pub numerical_enum : Option < serde_json
    :: Value >, # [serde(rename = "oneOf")] pub one_of : Option < SchemaArray
    >, pub pattern : Option < String >, # [serde(default)] #
    [serde(rename = "patternProperties")] pub pattern_properties : :: std ::
    collections :: BTreeMap < String, Schema >, # [serde(default)] pub
    properties : :: std :: collections :: BTreeMap < String, Schema >, pub
    required : Option < StringArray >, pub title : Option < String >, #
    [serde(default)] # [serde(with = "::schemafy_core::one_or_many")] #
    [serde(rename = "type")] pub type_ : Vec < SimpleTypes >, #
    [serde(rename = "uniqueItems")] pub unique_items : Option < bool >
}