// Moysis Moysis Volos, Greece 29/06/2024.

use quote::quote;
use syn::{Error, Field, LitStr, Result, Type};

// Constants representing the names of different metric types.
const COUNTER_TY: &str = "Counter"; // Represents the Counter metric type.
const HISTOGRAM_TY: &str = "Histogram"; // Represents the Histogram metric type.
const GAUGE_TY: &str = "Gauge"; // Represents the Gauge metric type.

/// Represents a metric with an associated field.
///
/// This struct holds information about a metric, including the field it is associated with,
/// its description, and an optional rename attribute.
pub(crate) struct Metric<'a> {
    pub(crate) field: &'a Field, // The field associated with the metric.
    pub(crate) description: String, // The description of the metric.
    rename: Option<LitStr>, // An optional rename attribute for the metric.
}

impl<'a> Metric<'a> {
    /// Creates a new `Metric` instance.
    ///
    /// # Arguments
    /// * `field` - A reference to the field associated with the metric.
    /// * `description` - A description of the metric.
    /// * `rename` - An optional rename attribute for the metric.
    ///
    /// # Returns
    /// A new `Metric` instance.
    pub(crate) const fn new(field: &'a Field, description: String, rename: Option<LitStr>) -> Self {
        Self { field, description, rename }
    }

    /// Returns the name of the metric.
    ///
    /// If a rename attribute is provided, it returns the renamed value.
    /// Otherwise, it returns the name of the field.
    pub(crate) fn name(&self) -> String {
        match self.rename.as_ref() {
            Some(name) => name.value(),
            None => self.field.ident.as_ref().map(ToString::to_string).unwrap_or_default(),
        }
    }

    /// Generates the registration statement for the metric.
    ///
    /// This method determines the appropriate registration macro based on the metric type.
    ///
    /// # Returns
    /// A `Result` containing a token stream with the registration statement, or an error if the metric type is unsupported.
    pub(crate) fn register_stmt(&self) -> Result<proc_macro2::TokenStream> {
        if let Type::Path(ref path_ty) = self.field.ty {
            if let Some(last) = path_ty.path.segments.last() {
                let registrar = match last.ident.to_string().as_str() {
                    COUNTER_TY => quote! { metrics::counter! },
                    HISTOGRAM_TY => quote! { metrics::histogram! },
                    GAUGE_TY => quote! { metrics::gauge! },
                    _ => return Err(Error::new_spanned(path_ty, "Unsupported metric type")),
                };

                return Ok(quote! { #registrar })
            }
        }

        Err(Error::new_spanned(&self.field.ty, "Unsupported metric type"))
    }

    /// Generates the description statement for the metric.
    ///
    /// This method determines the appropriate description macro based on the metric type.
    ///
    /// # Returns
    /// A `Result` containing a token stream with the description statement, or an error if the metric type is unsupported.
    pub(crate) fn describe_stmt(&self) -> Result<proc_macro2::TokenStream> {
        if let Type::Path(ref path_ty) = self.field.ty {
            if let Some(last) = path_ty.path.segments.last() {
                let descriptor = match last.ident.to_string().as_str() {
                    COUNTER_TY => quote! { metrics::describe_counter! },
                    HISTOGRAM_TY => quote! { metrics::describe_histogram! },
                    GAUGE_TY => quote! { metrics::describe_gauge! },
                    _ => return Err(Error::new_spanned(path_ty, "Unsupported metric type")),
                };

                return Ok(quote! { #descriptor })
            }
        }

        Err(Error::new_spanned(&self.field.ty, "Unsupported metric type"))
    }
}