use utoipa::OpenApi;

use crate::handlers;

pub struct SecurityAddon;
impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(
                        utoipa::openapi::security::ApiKeyValue::new("VLADIVOSTOK85000"),
                    ),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::courses::get_all_courses,
        handlers::courses::get_courses_by_ids,
        handlers::courses::get_course,
        handlers::courses::get_course_progress,
        handlers::courses::get_user_courses,
        handlers::courses::get_user_courses_started,
        handlers::courses::get_user_courses_completed,
        handlers::modules::get_modules_for_course,
        handlers::modules::get_module,
        handlers::courses::add_course_to_favourite,
        handlers::courses::get_favourite_courses,
        handlers::tasks::get_tasks_for_module,
        handlers::tasks::get_task,
        handlers::tasks::submit_task,
        handlers::tasks::task_progress,
        
        handlers::certs::get_certs,
        handlers::certs::create_cert,
        handlers::certs::get_cert_file

    ),
    // components(
    //     schemas(UserLogin, ErrorResponse, TokensPayload)
    // ),
    modifiers(&SecurityAddon),
    tags(
        (name = "NeoTeo-Courses", description = "Один статус код может обозначать несколько ошибок, обозначены через ;. Ошибка 5xx обозначает непредвиденную ошибку. В таком случае в error_message содержится текст ошибки Раста. ЕЩЕ!!!!!! Стоит отметить, что все возвращаемые джсоны находятся в поле data, т.е data: CourseProgress struct")
    )
)]
pub struct ApiDoc;
