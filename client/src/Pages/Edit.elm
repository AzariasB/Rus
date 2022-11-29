module Pages.Edit exposing (ExternalMsg(..), Model, Msg(..), createLink, editLink, init, update, view)

import Html exposing (Html, a, button, div, input, text)
import Html.Attributes as A
import Html.Events as E
import Http
import Json.Decode exposing (Decoder, bool, field, map2, string)
import Session



-- Model


type Status
    = CreatingUrl
    | SubmittingUrl
    | Error String


type alias Model =
    { status : Status, input : String, session : Session.Data, submitter : String -> Cmd ExternalMsg }


type InternalMsg
    = ClickedSubmit
    | TypedLink String


type ExternalMsg
    = EditJson (Result Http.Error EditRespnose)
    | CreateLinkError String


type Msg
    = Internal InternalMsg
    | External ExternalMsg



-- init


init : Session.Data -> ( String, String -> Cmd ExternalMsg ) -> ( Model, Cmd Msg )
init session ( baseData, submitter ) =
    ( { status = CreatingUrl
      , input = baseData
      , session = session
      , submitter = submitter
      }
    , Cmd.none
    )



-- VIEW


view : Model -> Html Msg
view model =
    let
        disabled =
            disableInput model.status
    in
    div [ A.class "container" ]
        [ div [ A.class "twelve columns" ]
            [ input
                [ A.type_ "text"
                , A.placeholder "Url to shorten"
                , A.autofocus (not disabled)
                , A.value model.input
                , A.disabled disabled
                , A.class "u-full-width"
                , E.onInput (TypedLink >> Internal)
                ]
                []
            ]
        , div [ A.class "twelve columns" ]
            [ div [ A.class "two columns" ]
                [ a [ A.href "/", A.class "button" ]
                    [ text "Cancel"
                    ]
                ]
            , div [ A.class "ten columns" ]
                [ button [ E.onClick <| Internal ClickedSubmit, A.disabled disabled ] [ text "Save" ]
                ]
            ]
        ]


disableInput : Status -> Bool
disableInput status =
    status == SubmittingUrl



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case ( msg, model.status ) of
        ( Internal ClickedSubmit, CreatingUrl ) ->
            ( model, Cmd.map External <| model.submitter model.input )

        ( Internal (TypedLink link), _ ) ->
            ( { model | status = CreatingUrl, input = link }, Cmd.none )

        ( External (CreateLinkError err), _ ) ->
            ( { model | status = Error err }, Cmd.none )

        ( _, _ ) ->
            ( model, Cmd.none )


createLink : String -> Cmd ExternalMsg
createLink model =
    Http.request
        { url = "/api/v1/redirections"
        , body = Http.stringBody "application/x-www-form-urlencoded" ("long_url=" ++ model)
        , expect = Http.expectJson EditJson editDecoder
        , method = "POST"
        , headers = []
        , timeout = Nothing
        , tracker = Nothing
        }


editLink : String -> String -> Cmd ExternalMsg
editLink short_url long_url =
    Http.request
        { url = "/api/v1/redirections/" ++ short_url
        , body = Http.stringBody "application/x-www-form-urlencoded" ("long_url=" ++ long_url)
        , expect = Http.expectJson EditJson editDecoder
        , method = "PUT"
        , headers = []
        , timeout = Nothing
        , tracker = Nothing
        }


type alias EditRespnose =
    { error : Bool
    , message : String
    }


editDecoder : Decoder EditRespnose
editDecoder =
    map2 EditRespnose
        (field "error" bool)
        (field "message" string)
