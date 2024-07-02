// Moysis Moysis Volos, Greece 29/06/2024.

// Importing the Lazy type from the once_cell crate for defining lazily initialized static variables.
use once_cell::sync::Lazy;

// Importing the quote macro and ToTokens trait from the quote crate for generating Rust code as tokens.
use quote::{quote, ToTokens};

// Importing the Regex type from the regex crate for regular expression handling.
use regex::Regex;

// Importing various types and traits from the syn crate for parsing and manipulating Rust code.
use syn::{
    punctuated::Punctuated, // For handling punctuated sequences in syntax trees.
    Attribute,              // Represents attributes in Rust code.
    Data,                   // Enum for different types of data structures (struct, enum, union).
    DeriveInput,            // Represents the entire input to a derive macro.
    Error,                  // Represents errors that can occur during parsing.
    Expr,                   // Represents Rust expressions.
    Field,                  // Represents a field in a struct or enum.
    Lit,                    // Represents literal values in Rust code.
    LitBool,                // Represents boolean literals.
    LitStr,                 // Represents string literals.
    Meta,                   // Represents meta items, such as attributes.
    MetaNameValue,          // Represents meta items in name-value form.
    Result,                 // A type alias for results specific to syn.
    Token,                  // Represents tokens in Rust code.
};

use crate::{metric::Metric, with_attrs::WithAttrs};

/// Metric name regex according to Prometheus data model
///
/// See <https://prometheus.io/docs/concepts/data_model/#metric-names-and-labels>
static METRIC_NAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z_:.][a-zA-Z0-9_:.]*$").unwrap());

/// Supported metrics separators
const SUPPORTED_SEPARATORS: &[&str] = &[".", "_", ":"];

/// Represents a field that can either be included as a metric or skipped.
///
/// This enum is used to differentiate between fields that should be processed
/// as metrics and those that should be ignored.
///
/// # Variants
///
/// * `Included` - A field that is included as a metric. Contains a `Metric<'a>`.
/// * `Skipped` - A field that is skipped. Contains a reference to a `Field`.
enum MetricField<'a> {
    /// A field that is included as a metric.
    Included(Metric<'a>),
    
    /// A field that is skipped and not processed as a metric.
    Skipped(&'a Field),
}

impl<'a> MetricField<'a> {
    /// Provides a reference to the underlying `Field`, whether the field is included as a metric or skipped.
    ///
    /// This method allows for consistent access to the `Field` within a `MetricField` instance,
    /// facilitating operations that need to interact with the field regardless of its inclusion status.
    ///
    /// # Arguments
    ///
    /// * `&self` - A reference to the current instance of `MetricField`.
    const fn field(&self) -> &'a Field {
        match self {
            MetricField::Included(Metric { field, .. }) | MetricField::Skipped(field) => field,
        }
    }
}

/// Derives the necessary implementation for a given `DeriveInput`.
///
/// This function generates code for metrics, including their registration,
/// description, and instantiation with or without labels. It handles both static
/// and dynamic metric scopes.
///
/// # Arguments
///
/// * `node` - A reference to the `DeriveInput` which represents the input to a `derive` macro.
///
/// * `Result<proc_macro2::TokenStream>` - The generated token stream or an error.
///
/// # Errors
/// This function will return an error if parsing metrics attributes or fields fails.
pub(crate) fn derive(node: &DeriveInput) -> Result<proc_macro2::TokenStream> {
    // Extract the identifier, visibility, and name of the type.
    let ty = &node.ident;
    let vis = &node.vis;
    let ident_name = ty.to_string();

    // Parse metrics attributes and fields from the node.
    let metrics_attr = parse_metrics_attr(node)?;
    let metric_fields = parse_metric_fields(node)?;

    // Documentation for the describe method.
    let describe_doc = quote! {
        /// Describe all exposed metrics. Internally calls `describe_*` macros from
        /// the metrics crate according to the metric type.
        ///
        /// See <https://docs.rs/metrics/0.20.1/metrics/index.html#macros>
    };

    // Generate code based on the scope of metrics (static or dynamic).
    let register_and_describe = match &metrics_attr.scope {
        MetricsScope::Static(scope) => {
            // Process fields for static scope.
            let (defaults, labeled_defaults, describes): (Vec<_>, Vec<_>, Vec<_>) = metric_fields
                .iter()
                .map(|metric| {
                    let field_name = &metric.field().ident;
                    match metric {
                        MetricField::Included(metric) => {
                            let metric_name = format!(
                                "{}{}{}",
                                scope.value(),
                                metrics_attr.separator(),
                                metric.name()
                            );
                            let registrar = metric.register_stmt()?;
                            let describe = metric.describe_stmt()?;
                            let description = &metric.description;
                            Ok((
                                quote! {
                                    #field_name: #registrar(#metric_name),
                                },
                                quote! {
                                    #field_name: #registrar(#metric_name, labels.clone()),
                                },
                                Some(quote! {
                                    #describe(#metric_name, #description);
                                }),
                            ))
                        }
                        MetricField::Skipped(_) => Ok((
                            quote! {
                                #field_name: Default::default(),
                            },
                            quote! {
                                #field_name: Default::default(),
                            },
                            None,
                        )),
                    }
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .fold((vec![], vec![], vec![]), |mut acc, x| {
                    acc.0.push(x.0);
                    acc.1.push(x.1);
                    if let Some(describe) = x.2 {
                        acc.2.push(describe);
                    }
                    acc
                });

            // Generate implementations for the static scope.
            quote! {
                impl Default for #ty {
                    fn default() -> Self {
                        #ty::describe();

                        Self {
                            #(#defaults)*
                        }
                    }
                }

                impl #ty {
                    /// Create new instance of metrics with provided labels.
                    #vis fn new_with_labels(labels: impl metrics::IntoLabels + Clone) -> Self {
                        Self {
                            #(#labeled_defaults)*
                        }
                    }

                    #describe_doc
                    #vis fn describe() {
                        #(#describes)*
                    }
                }
            }
        }
        MetricsScope::Dynamic => {
            // Process fields for dynamic scope.
            let (defaults, labeled_defaults, describes): (Vec<_>, Vec<_>, Vec<_>) = metric_fields
                .iter()
                .map(|metric| {
                    let field_name = &metric.field().ident;
                    match metric {
                        MetricField::Included(metric) => {
                            let name = metric.name();
                            let separator = metrics_attr.separator();
                            let metric_name = quote! {
                                format!("{}{}{}", scope, #separator, #name)
                            };

                            let registrar = metric.register_stmt()?;
                            let describe = metric.describe_stmt()?;
                            let description = &metric.description;

                            Ok((
                                quote! {
                                    #field_name: #registrar(#metric_name),
                                },
                                quote! {
                                    #field_name: #registrar(#metric_name, labels.clone()),
                                },
                                Some(quote! {
                                    #describe(#metric_name, #description);
                                }),
                            ))
                        }
                        MetricField::Skipped(_) => Ok((
                            quote! {
                                #field_name: Default::default(),
                            },
                            quote! {
                                #field_name: Default::default(),
                            },
                            None,
                        )),
                    }
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .fold((vec![], vec![], vec![]), |mut acc, x| {
                    acc.0.push(x.0);
                    acc.1.push(x.1);
                    if let Some(describe) = x.2 {
                        acc.2.push(describe);
                    }
                    acc
                });

            // Generate implementations for the dynamic scope.
            quote! {
                impl #ty {
                    /// Create new instance of metrics with provided scope.
                    #vis fn new(scope: &str) -> Self {
                        #ty::describe(scope);

                        Self {
                            #(#defaults)*
                        }
                    }

                    /// Create new instance of metrics with provided labels.
                    #vis fn new_with_labels(scope: &str, labels: impl metrics::IntoLabels + Clone) -> Self {
                        Self {
                            #(#labeled_defaults)*
                        }
                    }

                    #describe_doc
                    #vis fn describe(scope: &str) {
                        #(#describes)*
                    }
                }
            }
        }
    };

    // Combine the generated implementations with a Debug implementation.
    Ok(quote! {
        #register_and_describe

        impl std::fmt::Debug for #ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#ident_name).finish()
            }
        }
    })
}

/// Represents the attributes for metrics.
///
/// This struct holds configuration details for metrics, including their scope and separator.
///
/// # Fields
/// * `scope` - Defines the scope of the metrics, which can be either static or dynamic.
/// * `separator` - An optional string literal used as a separator in metric names.
pub(crate) struct MetricsAttr {
    pub(crate) scope: MetricsScope,      // The scope of the metrics.
    pub(crate) separator: Option<LitStr>, // Optional separator for metric names.
}

impl MetricsAttr {
    /// The default separator used in metric names when no separator is specified.
    const DEFAULT_SEPARATOR: &'static str = ".";

    /// Returns the separator to be used in metric names.
    ///
    /// This method checks if a custom separator is specified in the `MetricsAttr`.
    /// If a custom separator is provided, it returns its value. Otherwise, it returns
    /// the default separator.
    ///
    /// A `String` representing the separator to be used in metric names.
    fn separator(&self) -> String {
        match &self.separator {
            Some(sep) => sep.value(), // Use the custom separator if provided.
            None => Self::DEFAULT_SEPARATOR.to_owned(), // Use the default separator.
        }
    }
}

/// Defines the scope for metrics, which can be either static or dynamic.
///
/// # Variants
/// * `Static` - A static scope with a fixed string literal. Metrics with this scope
///   use a predefined scope value provided by `LitStr`.
/// * `Dynamic` - A dynamic scope. Metrics with this scope do not use a predefined
///   value and can be dynamically scoped at runtime.
pub(crate) enum MetricsScope {
    /// A static scope with a fixed string literal.
    Static(LitStr),

    /// A dynamic scope without a predefined value.
    Dynamic,
}
/// Parses the `metrics` attribute from the given `DeriveInput`.
///
/// This function extracts and validates the `metrics` attribute, ensuring that it contains
/// the necessary information for defining metrics, such as scope and separator.
///
/// # Arguments
/// * `node` - A reference to the `DeriveInput` which represents the input to a derive macro.
///
/// # Returns
/// A `Result` containing the parsed `MetricsAttr` or an error if the parsing fails.
///
/// # Errors
/// This function will return an error if the `metrics` attribute is malformed, contains
/// duplicate entries, unsupported values, or if required values are missing.
fn parse_metrics_attr(node: &DeriveInput) -> Result<MetricsAttr> {
    // Parse the required "metrics" attribute.
    let metrics_attr = parse_single_required_attr(node, "metrics")?;
    
    // Parse the arguments of the attribute.
    let parsed =
        metrics_attr.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?;
    
    // Initialize variables to store attribute values.
    let (mut scope, mut separator, mut dynamic) = (None, None, None);
    
    // Iterate over the parsed key-value pairs.
    for kv in parsed {
        let lit = match kv.value {
            Expr::Lit(ref expr) => &expr.lit,
            _ => continue,
        };
        
        // Check for the "scope" key.
        if kv.path.is_ident("scope") {
            if scope.is_some() {
                return Err(Error::new_spanned(kv, "Duplicate `scope` value provided."));
            }
            let scope_lit = parse_str_lit(lit)?;
            validate_metric_name(&scope_lit)?;
            scope = Some(scope_lit);
        
        // Check for the "separator" key.
        } else if kv.path.is_ident("separator") {
            if separator.is_some() {
                return Err(Error::new_spanned(kv, "Duplicate `separator` value provided."));
            }
            let separator_lit = parse_str_lit(lit)?;
            if !SUPPORTED_SEPARATORS.contains(&&*separator_lit.value()) {
                return Err(Error::new_spanned(
                    kv,
                    format!(
                        "Unsupported `separator` value. Supported: {}.",
                        SUPPORTED_SEPARATORS
                            .iter()
                            .map(|sep| format!("`{sep}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                ));
            }
            separator = Some(separator_lit);
        
        // Check for the "dynamic" key.
        } else if kv.path.is_ident("dynamic") {
            if dynamic.is_some() {
                return Err(Error::new_spanned(kv, "Duplicate `dynamic` flag provided."));
            }
            dynamic = Some(parse_bool_lit(lit)?.value);
        
        // Handle unsupported keys.
        } else {
            return Err(Error::new_spanned(kv, "Unsupported attribute entry."));
        }
    }

    // Determine the scope based on parsed values.
    let scope = match (scope, dynamic) {
        (Some(scope), None) | (Some(scope), Some(false)) => MetricsScope::Static(scope),
        (None, Some(true)) => MetricsScope::Dynamic,
        (Some(_), Some(_)) => {
            return Err(Error::new_spanned(node, "`scope = ..` conflicts with `dynamic = true`."));
        }
        _ => {
            return Err(Error::new_spanned(
                node,
                "Either `scope = ..` or `dynamic = true` must be set.",
            ));
        }
    };

    // Return the parsed MetricsAttr.
    Ok(MetricsAttr { scope, separator })
}

/// Parses the metric fields from the given `DeriveInput`.
///
/// This function extracts and validates the `metric` attributes from the fields of a struct,
/// ensuring that they contain the necessary information for defining metrics, such as description
/// and renaming options. Only structs are supported.
///
/// # Arguments
/// * `node` - A reference to the `DeriveInput` which represents the input to a derive macro.
///
/// # Returns
/// A `Result` containing a vector of `MetricField` or an error if the parsing fails.
///
/// # Errors
/// This function will return an error if the input is not a struct, if the `metric` attributes are
/// malformed, contain duplicate entries, unsupported values, or if required values are missing.
fn parse_metric_fields(node: &DeriveInput) -> Result<Vec<MetricField<'_>>> {
    // Ensure the input is a struct.
    let Data::Struct(ref data) = node.data else {
        return Err(Error::new_spanned(node, "Only structs are supported."))
    };

    // Prepare a vector to store parsed metric fields.
    let mut metrics = Vec::with_capacity(data.fields.len());
    
    // Iterate over the fields of the struct.
    for field in &data.fields {
        let (mut describe, mut rename, mut skip) = (None, None, false);
        
        // Parse the `metric` attribute if it exists.
        if let Some(metric_attr) = parse_single_attr(field, "metric")? {
            let parsed =
                metric_attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            
            // Iterate over the parsed key-value pairs.
            for meta in parsed {
                match meta {
                    Meta::Path(path) if path.is_ident("skip") => skip = true,
                    Meta::NameValue(kv) => {
                        let lit = match kv.value {
                            Expr::Lit(ref expr) => &expr.lit,
                            _ => continue,
                        };
                        
                        // Check for the "describe" key.
                        if kv.path.is_ident("describe") {
                            if describe.is_some() {
                                return Err(Error::new_spanned(
                                    kv,
                                    "Duplicate `describe` value provided.",
                                ))
                            }
                            describe = Some(parse_str_lit(lit)?);
                        
                        // Check for the "rename" key.
                        } else if kv.path.is_ident("rename") {
                            if rename.is_some() {
                                return Err(Error::new_spanned(
                                    kv,
                                    "Duplicate `rename` value provided.",
                                ))
                            }
                            let rename_lit = parse_str_lit(lit)?;
                            validate_metric_name(&rename_lit)?;
                            rename = Some(rename_lit)
                        
                        // Handle unsupported keys.
                        } else {
                            return Err(Error::new_spanned(kv, "Unsupported attribute entry."))
                        }
                    }
                    _ => return Err(Error::new_spanned(meta, "Unsupported attribute entry.")),
                }
            }
        }

        // Handle skipped fields.
        if skip {
            metrics.push(MetricField::Skipped(field));
            continue
        }

        // Determine the description for the metric.
        let description = match describe {
            Some(lit_str) => lit_str.value(),
            // Parse docs only if `describe` attribute was not provided.
            None => match parse_docs_to_string(field)? {
                Some(docs_str) => docs_str,
                None => {
                    return Err(Error::new_spanned(
                        field,
                        "Either doc comment or `describe = ..` must be set.",
                    ))
                }
            },
        };

        // Add the included metric to the vector.
        metrics.push(MetricField::Included(Metric::new(field, description, rename)));
    }

    // Return the parsed metric fields.
    Ok(metrics)
}

/// Validates that the given metric name matches a predefined regex pattern.
///
/// This function checks if the provided metric name conforms to the expected format,
/// ensuring that it adheres to a specific regular expression. If the name does not
/// match the regex, an error is returned.
///
/// # Arguments
/// * `name` - A reference to a `LitStr` containing the metric name to be validated.
///
/// # Returns
/// A `Result` indicating success if the name matches the regex, or an error if it does not.
///
/// # Errors
/// This function returns an error if the metric name does not match the predefined regex pattern.
fn validate_metric_name(name: &LitStr) -> Result<()> {
    if METRIC_NAME_RE.is_match(&name.value()) {
        Ok(())
    } else {
        Err(Error::new_spanned(name, format!("Value must match regex {}", METRIC_NAME_RE.as_str())))
    }
}

/// Parses a single attribute with the specified identifier from the given token.
///
/// This function searches for an attribute with the given identifier in the attributes
/// of the provided token. It ensures that at most one such attribute exists. If multiple
/// attributes with the same identifier are found, an error is returned.
///
/// # Arguments
/// * `token` - A reference to a type that implements `WithAttrs` and `ToTokens`.
/// * `ident` - A string slice containing the identifier of the attribute to be parsed.
///
/// # Returns
/// A `Result` containing an `Option` with a reference to the found attribute, or `None`
/// if no such attribute is found.
///
/// # Errors
/// This function returns an error if multiple attributes with the same identifier are found.
fn parse_single_attr<'a, T: WithAttrs + ToTokens>(
    token: &'a T,
    ident: &str,
) -> Result<Option<&'a Attribute>> {
    // Iterate over the attributes of the token and filter by the specified identifier.
    let mut attr_iter = token.attrs().iter().filter(|a| a.path().is_ident(ident));
    
    // Check if there is a matching attribute.
    if let Some(attr) = attr_iter.next() {
        // Ensure that there are no additional matching attributes.
        if let Some(next_attr) = attr_iter.next() {
            // Return an error if a duplicate attribute is found.
            Err(Error::new_spanned(
                next_attr,
                format!("Duplicate `#[{ident}(..)]` attribute provided."),
            ))
        } else {
            // Return the found attribute.
            Ok(Some(attr))
        }
    } else {
        // Return None if no matching attribute is found.
        Ok(None)
    }
}

/// Parses a single required attribute with the specified identifier from the given token.
///
/// This function searches for an attribute with the given identifier in the attributes
/// of the provided token. It ensures that exactly one such attribute exists. If no such
/// attribute or multiple attributes with the same identifier are found, an error is returned.
///
/// # Arguments
/// * `token` - A reference to a type that implements `WithAttrs` and `ToTokens`.
/// * `ident` - A string slice containing the identifier of the attribute to be parsed.
///
/// # Returns
/// A `Result` containing a reference to the found attribute.
///
/// # Errors
/// This function returns an error if no attribute with the specified identifier is found,
/// or if multiple attributes with the same identifier are found.
fn parse_single_required_attr<'a, T: WithAttrs + ToTokens>(
    token: &'a T,
    ident: &str,
) -> Result<&'a Attribute> {
    // Attempt to parse a single attribute with the specified identifier.
    if let Some(attr) = parse_single_attr(token, ident)? {
        // Return the found attribute.
        Ok(attr)
    } else {
        // Return an error if no such attribute is found.
        Err(Error::new_spanned(token, format!("`#[{ident}(..)]` attribute must be provided.")))
    }
}

/// Parses the documentation comments from the given token and concatenates them into a single string.
///
/// This function iterates over the attributes of the provided token, extracts documentation
/// comments, and concatenates them into a single string. It handles multiple lines of
/// documentation by concatenating them with a space.
///
/// # Arguments
/// * `token` - A reference to a type that implements `WithAttrs`.
///
/// # Returns
/// A `Result` containing an `Option` with the concatenated documentation string if any
/// documentation comments are found, or `None` if no documentation comments are present.
///
/// # Errors
/// This function does not return errors under normal circumstances, but it uses the `Result`
/// type to align with other parsing functions.
fn parse_docs_to_string<T: WithAttrs>(token: &T) -> Result<Option<String>> {
    let mut doc_str = None;
    
    // Iterate over the attributes of the token.
    for attr in token.attrs() {
        if let syn::Meta::NameValue(ref meta) = attr.meta {
            if let Expr::Lit(ref lit) = meta.value {
                if let Lit::Str(ref doc) = lit.lit {
                    let doc_value = doc.value().trim().to_string();
                    
                    // Concatenate the documentation string with previous values if present.
                    doc_str = Some(
                        doc_str
                            .map(|prev_doc_value| format!("{prev_doc_value} {doc_value}"))
                            .unwrap_or(doc_value),
                    );
                }
            }
        }
    }
    
    // Return the concatenated documentation string, if any.
    Ok(doc_str)
}

/// Parses a string literal from the given `Lit` expression.
///
/// This function checks if the provided literal is a string literal (`LitStr`). If it is,
/// the function returns the string literal. Otherwise, it returns an error.
///
/// # Arguments
/// * `lit` - A reference to a `Lit` (literal) expression.
///
/// # Returns
/// A `Result` containing the parsed `LitStr` if the input is a string literal.
///
/// # Errors
/// This function returns an error if the provided literal is not a string literal.
fn parse_str_lit(lit: &Lit) -> Result<LitStr> {
    match lit {
        // If the literal is a string literal, return it.
        Lit::Str(lit_str) => Ok(lit_str.to_owned()),
        // Otherwise, return an error indicating that a string literal was expected.
        _ => Err(Error::new_spanned(lit, "Value **must** be a string literal.")),
    }
}

/// Parses a boolean literal from the given `Lit` expression.
///
/// This function checks if the provided literal is a boolean literal (`LitBool`). If it is,
/// the function returns the boolean literal. Otherwise, it returns an error.
///
/// # Arguments
/// * `lit` - A reference to a `Lit` (literal) expression.
///
/// # Returns
/// A `Result` containing the parsed `LitBool` if the input is a boolean literal.
///
/// # Errors
/// This function returns an error if the provided literal is not a boolean literal.
fn parse_bool_lit(lit: &Lit) -> Result<LitBool> {
    match lit {
        // If the literal is a boolean literal, return it.
        Lit::Bool(lit_bool) => Ok(lit_bool.to_owned()),
        // Otherwise, return an error indicating that a boolean literal was expected.
        _ => Err(Error::new_spanned(lit, "Value **must** be a boolean literal.")),
    }
}