use api_models::payments::Card;
use serde::{Deserialize, Serialize};
use cards::CardNumber;
use masking::Secret;
use crate::{connector::utils::{self, PaymentsAuthorizeRequestData, RouterData},core::errors,types::{self,api, storage::enums::{self, Currency}}};


// Auth Struct
pub struct StripeAuthType {
    pub(super) apikey: Secret<String>,
}

impl TryFrom<&types::ConnectorAuthType> for stripeAuthType {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(auth_type: &types::ConnectorAuthType) -> Result<Self, Self::Error> {
        match auth_type {
            types::ConnectorAuthType::HeaderKey { api_key } => Ok(Self {
                apikey: api_key.to_owned(),
            }),
            _ => Err(errors::ConnectorError::FailedToObtainAuthType.into()),
        }
    }
}


#[derive(Debug, Serialize)]
pub struct stripeRouterData<T> {
    pub amount: i64,
    pub router_data: T,
}
impl<T>
    TryFrom<(
        &types::api::CurrencyUnit,
        types::storage::enums::Currency,
        i64,
        T,
    )> for stripeRouterData<T>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        (_currency_unit, _currency, amount, item): (
            &types::api::CurrencyUnit,
            types::storage::enums::Currency,
            i64,
            T,
        ),
    ) -> Result<Self, Self::Error> {
        
        Ok(Self {
            amount,
            router_data: item,
        })
    }
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct stripeAuthorizeRequestCard {
    pub number: CardNumber,
    pub exp_month: Secret<String>,
    pub exp_year: Secret<String>,
    pub cvc: Secret<String>,
    pub cardholder_name: Secret<String>,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct stripeAuthorizeRequest {
    pub amount: String,
    pub currency: diesel_models::enums::Currency,
    pub card: stripeAuthorizeRequestCard,
    pub captured: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct stripeAuthorizeResponseCard {
    pub id: String,
    pub created: i64,
    pub object_type: String,
    pub first6: String,
    pub last4: String,
    pub fingerprint: String,
    pub exp_month: Secret<String>,
    pub exp_year: Secret<String>,
    pub cardholder_name: Secret<String>,
    pub brand: String,
    #[serde(rename = "type")]
    pub stripe_authorize_response_card_type: String,
    pub country: api_models::enums::CountryAlpha2,
    pub issuer: String,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct stripeAuthorizeResponseFraudDetails {
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct stripeAuthorizeResponseAvsCheck {
    pub result: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct stripeAuthorizeResponse {
    pub id: String,
    pub created: i64,
    pub object_type: String,
    pub amount: String,
    pub amount_refunded: i64,
    pub currency: diesel_models::enums::Currency,
    pub card: stripeAuthorizeResponseCard,
    pub captured: String,
    pub refunded: bool,
    pub disputed: bool,
    pub fraud_details: stripeAuthorizeResponseFraudDetails,
    pub avs_check: stripeAuthorizeResponseAvsCheck,
    pub status: stripeAttemptStatus,
    pub client_object_id: String,
}

impl TryFrom<(&stripeRouterData<&types::PaymentsAuthorizeRouterData>, &Card)> for stripeAuthorizeRequest {
            type Error = error_stack::Report<errors::ConnectorError>;
            fn try_from(value: (&stripeRouterData<&types::PaymentsAuthorizeRouterData>, &Card)) -> Result<Self, Self::Error> {
                let (item, ccard) = value;
                let stripe_authorize_request_card = StripeAuthorizeRequestCard{number:ccard.card_number.clone(),exp_month:ccard.card_exp_month.clone(),exp_year:ccard.card_exp_year.clone(),cvc:ccard.card_cvc.clone(),cardholder_name:ccard.card_holder_name.clone()};
			let stripe_authorize_request = StripeAuthorizeRequest{amount:item.amount,currency:item.router_data.request.currency,card:stripe_authorize_request_card,captured:item.router_data.request.is_auto_capture()?};
                Ok(stripe_authorize_request)
            }
        }    
impl TryFrom<&stripeRouterData<&types::PaymentsAuthorizeRouterData>> for stripeAuthorizeRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &stripeRouterData<&types::PaymentsAuthorizeRouterData>) -> Result<Self, Self::Error> {
        match &item.router_data.request.payment_method_data {
            api_models::payments::PaymentMethodData::Card(card) => Self::try_from((item, card)),
            _ => Err(errors::ConnectorError::NotImplemented(
                "payment method".to_string(),
            ))?,
        }
    }
}

impl TryFrom<types::PaymentsResponseRouterData<stripeAuthorizeResponse>> 
    for types::PaymentsAuthorizeRouterData
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: types::PaymentsResponseRouterData<stripeAuthorizeResponse>,
    ) -> Result<Self,Self::Error> {
        Ok(Self {
            status: enums::AttemptStatus::from(item.response.status),
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.id),
                redirection_data:  None,
                mandate_reference: None,
                connector_metadata: None,
                network_txn_id: None,
                connector_response_reference_id: None,
            }),
            ..item.data
        })
    }
}
#[derive(Debug, Serialize, Deserialize)]

pub enum stripeAttemptStatus {
    Successful,
	Failed,
	Pending
}
impl From<StripeAttemptStatus> for enums::AttemptStatus {
    fn from(item: StripeAttemptStatus) -> Self {
        match item {
            StripeAttemptStatus::Successful => Self::Charged,
			StripeAttemptStatus::Failed => Self::Failure,
			StripeAttemptStatus::Pending => Self::Pending
        }
    }
}


//TODO: Fill the struct with respective fields
// REFUND :
// Type definition for RefundRequest
#[derive(Default, Debug, Serialize)]
pub struct RefundRequest {
    pub amount: i64
}

impl<F> TryFrom<&types::RefundsRouterData<F>> for RefundRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::RefundsRouterData<F>) -> Result<Self,Self::Error> {
        Ok(Self {
            amount: item.request.refund_amount,
        })
    }
}

// Type definition for Refund Response

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub enum RefundStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
}

impl From<RefundStatus> for enums::RefundStatus {
    fn from(item: RefundStatus) -> Self {
        match item {
            RefundStatus::Succeeded => Self::Success,
            RefundStatus::Failed => Self::Failure,
            RefundStatus::Processing => Self::Pending,
            //TODO: Review mapping
        }
    }
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    id: String,
    status: RefundStatus
}

impl TryFrom<types::RefundsResponseRouterData<api::Execute, RefundResponse>>
    for types::RefundsRouterData<api::Execute>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: types::RefundsResponseRouterData<api::Execute, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(types::RefundsResponseData {
                connector_refund_id: item.response.id.to_string(),
                refund_status: enums::RefundStatus::from(item.response.status),
            }),
            ..item.data
        })
    }
}

impl TryFrom<types::RefundsResponseRouterData<api::RSync, RefundResponse>> for types::RefundsRouterData<api::RSync>
{
     type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: types::RefundsResponseRouterData<api::RSync, RefundResponse>) -> Result<Self,Self::Error> {
        Ok(Self {
            response: Ok(types::RefundsResponseData {
                connector_refund_id: item.response.id.to_string(),
                refund_status: enums::RefundStatus::from(item.response.status),
            }),
            ..item.data
        })
     }
 }

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct ErrorResponse {
    pub status_code: u16,
    pub code: String,
    pub message: String,
    pub reason: Option<String>,
}
