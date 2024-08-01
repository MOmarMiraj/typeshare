package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class DefinedTwice (
	val field1: String
)

@Serializable
object Excluded

@Serializable
object MultipleTargets

@Serializable
object OtherExcluded

@Serializable
enum class SomeEnum(val string: String) {
}

/// Generated type representing the anonymous struct variant `Variant7` of the `TestEnum` Rust enum
@Serializable
data class TestEnumVariant7Inner (
	val field1: String
)

@Serializable
sealed class TestEnum {
	@Serializable
	@SerialName("Variant1")
	object Variant1: TestEnum()
	@Serializable
	@SerialName("Variant5")
	object Variant5: TestEnum()
	@Serializable
	@SerialName("Variant7")
	data class Variant7(val content: TestEnumVariant7Inner): TestEnum()
	@Serializable
	@SerialName("Variant8")
	object Variant8: TestEnum()
}

