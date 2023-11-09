
pub mod transformers;

use std::fmt::Debug;

use error_stack::{IntoReport, ResultExt};
use masking::ExposeInterface;
use transformers as stripe;

use diesel_models::enums;
use crate::{
    configs::settings,
    core::errors::{self, CustomResult},
    headers,
    services::{
        self,
        request::{self, Mask},
        ConnectorIntegration, ConnectorValidation
    },
    types::{
        self,
        api::{self, ConnectorCommon, ConnectorCommonExt},
        ErrorResponse, Response,
    },
    utils::{self, BytesExt},
};

#[derive(Debug, Clone)]
pub struct Stripe;

impl api::Payment for Stripe {}
impl api::PaymentSession for Stripe {}
impl api::ConnectorAccessToken for Stripe {}
impl api::MandateSetup for Stripe {}
impl api::PaymentAuthorize for Stripe {}
impl api::PaymentSync for Stripe {}
impl api::PaymentCapture for Stripe {}
impl api::PaymentVoid for Stripe {}
impl api::Refund for Stripe {}
impl api::RefundExecute for Stripe {}
impl api::RefundSync for Stripe {}
impl api::PaymentToken for Stripe {}
impl<Flow, Request, Response> ConnectorCommonExt<Flow, Request, Response> for Stripe
where
    Self: ConnectorIntegration<Flow, Request, Response>,
{
    fn build_headers(
        &self,
        req: &types::RouterData<Flow, Request, Response>,
        _connectors: &settings::Connectors,
    ) -> CustomResult<Vec<(String, request::Maskable<String>)>, errors::ConnectorError> {

        let mut headers = vec![
            (
                headers::CONTENT_TYPE.to_string(),
                Self::get_content_type(self).to_string().into(),
            )
        ];
        let mut api_key = self.get_auth_header(&req.connector_auth_type)?;
        headers.append(&mut api_key);
        Ok(headers)
    }
}
impl ConnectorCommon for Stripe {
    fn id(&self) -> &'static str {
        "stripe"
    }

    fn common_get_content_type(&self) -> &'static str {
        "application/json"
    }

    fn base_url<'a>(&self, connectors: &'a settings::Connectors) -> &'a str {
        connectors.stripe.base_url.as_ref()
    }

    fn get_auth_header(
        &self,
        auth_type: &types::ConnectorAuthType,
    ) -> CustomResult<Vec<(String, request::Maskable<String>)>, errors::ConnectorError> {
        let auth = stripe::StripeAuthType::try_from(auth_type)
            .change_context(errors::ConnectorError::FailedToObtainAuthType)?;
        Ok(vec![(
            headers::AUTHORIZATION.to_string(),
            auth.apikey.expose().into_masked(),
        )])
    }
}

impl ConnectorValidation for Stripe {
    fn validate_capture_method(
        &self,
        capture_method: Option<enums::CaptureMethod>,
    ) -> CustomResult<(), errors::ConnectorError> {
        let capture_method = capture_method.unwrap_or_default();
        match capture_method {
            enums::CaptureMethod::Automatic | enums::CaptureMethod::Manual => Ok(()),
            enums::CaptureMethod::ManualMultiple | enums::CaptureMethod::Scheduled => Err(
                super::utils::construct_not_supported_error_report(capture_method, self.id()),
            ),
        }
    }
}
impl ConnectorIntegration<api::PaymentMethodToken, types::PaymentMethodTokenizationData, types::PaymentsResponseData> for Stripe {
}


impl ConnectorIntegration<api::AccessTokenAuth, types::AccessTokenRequestData, types::AccessToken> for Stripe {
}


impl ConnectorIntegration<api::SetupMandate, types::SetupMandateRequestData, types::PaymentsResponseData> for Stripe {
}


impl ConnectorIntegration<api::Authorize, types::PaymentsAuthorizeData, types::PaymentsResponseData> for Stripe {
    fn get_headers(
        &self,
        req: &types::PaymentsAuthorizeRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Vec<(String, request::Maskable<String>)>, errors::ConnectorError> {
        self.build_headers(req, connectors)
    }
    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }
    fn get_url(&self, req: &types::PaymentsAuthorizeRouterData, connectors: &settings::Connectors,) -> CustomResult<String,errors::ConnectorError> {
            Ok(format!(
                "{}/charges",
                self.base_url(connectors)
            ))
    }
    fn get_request_body(&self, req: &types::PaymentsAuthorizeRouterData) -> CustomResult<Option<types::RequestBody>, errors::ConnectorError> {
        let connector_router_data = stripe::StripeRouterData::try_from((
            &self.get_currency_unit(),
            req.request.currency,
            req.request.amount,
            req,
        ))?;
        let req_obj = stripe::StripeAuthorizeRequest::try_from(&connector_router_data)?;
        let stripe_req = types::RequestBody::log_and_get_request_body(&req_obj, utils::Encode::<stripe::StripeAuthorizeRequest>::encode_to_string_of_json)
            .change_context(errors::ConnectorError::RequestEncodingFailed)?;
        Ok(Some(stripe_req))
    }
    fn build_request(
        &self,
        req: &types::PaymentsAuthorizeRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        Ok(Some(
            services::RequestBuilder::new()
                .method(services::Method::Post)
                .url(&types::PaymentsAuthorizeType::get_url(
                    self, req, connectors,
                )?)
                .attach_default_headers()
                .headers(types::PaymentsAuthorizeType::get_headers(
                    self, req, connectors,
                )?)
                .body(types::PaymentsAuthorizeType::get_request_body(self, req)?)
                .build(),
        ))
    }
    fn handle_response(
        &self,
        data: &types::PaymentsAuthorizeRouterData,
        res: Response,
    ) -> CustomResult<types::PaymentsAuthorizeRouterData,errors::ConnectorError> {
        let response: stripe::StripeAuthorizeResponse = res.response.parse_struct("StripeAuthorizeResponse").change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        types::RouterData::try_from(types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
    }
    fn get_error_response(&self, res: Response) -> CustomResult<ErrorResponse,errors::ConnectorError> {
        self.build_error_response(res)
    }
}


impl ConnectorIntegration<api::Void, types::PaymentsCancelData, types::PaymentsResponseData> for Stripe {
}


impl ConnectorIntegration<api::PSync, types::PaymentsSyncData, types::PaymentsResponseData> for Stripe {
}


impl ConnectorIntegration<api::Capture, types::PaymentsCaptureData, types::PaymentsResponseData> for Stripe {
}


impl ConnectorIntegration<api::Session, types::PaymentsSessionData, types::PaymentsResponseData> for Stripe {
}


impl ConnectorIntegration<api::Execute, types::RefundsData, types::RefundsResponseData> for Stripe {
}


impl ConnectorIntegration<api::RSync, types::RefundsData, types::RefundsResponseData> for Stripe {
}


#[async_trait::async_trait]
impl api::IncomingWebhook for Stripe {
    fn get_webhook_object_reference_id(
        &self,
        _request: &api::IncomingWebhookRequestDetails<'_>,
    ) -> CustomResult<api::webhooks::ObjectReferenceId, errors::ConnectorError> {
        Err(errors::ConnectorError::WebhooksNotImplemented).into_report()
    }

    fn get_webhook_event_type(
        &self,
        _request: &api::IncomingWebhookRequestDetails<'_>,
    ) -> CustomResult<api::IncomingWebhookEvent, errors::ConnectorError> {
        Err(errors::ConnectorError::WebhooksNotImplemented).into_report()
    }

    fn get_webhook_resource_object(
        &self,
        _request: &api::IncomingWebhookRequestDetails<'_>,
    ) -> CustomResult<serde_json::Value, errors::ConnectorError> {
        Err(errors::ConnectorError::WebhooksNotImplemented).into_report()
    }
}