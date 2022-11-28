module Pages.Create exposing (ExternalMsg(..), Model, Msg(..), init, update, view)

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
    { status : Status, input : String, session : Session.Data }


type InternalMsg
    = ClickedCreateLink
    | TypedLink String


type ExternalMsg
    = CreatedLink (Result Http.Error CreateResponse)
    | CreateLinkError String


type Msg
    = Internal InternalMsg
    | External ExternalMsg



-- init


init : Session.Data -> ( Model, Cmd Msg )
init session =
    ( { status = CreatingUrl, input = "", session = session }, Cmd.none )



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
                [ button [ E.onClick <| Internal ClickedCreateLink, A.disabled disabled ] [ text "Create redirection" ]
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
        ( Internal ClickedCreateLink, CreatingUrl ) ->
            ( model, Cmd.map External <| createLink model.input )

        ( Internal (TypedLink _), _ ) ->
            ( { model | status = CreatingUrl }, Cmd.none )

        ( External (CreateLinkError err), _ ) ->
            ( { model | status = Error err }, Cmd.none )

        ( _, _ ) ->
            ( model, Cmd.none )


createLink : String -> Cmd ExternalMsg
createLink model =
    Http.request
        { url = "/api/v1/redirections"
        , body = Http.stringBody "application/x-www-form-urlencoded" ("long_url=" ++ model)
        , expect = Http.expectJson CreatedLink createDecoder
        , method = "POST"
        , headers = []
        , timeout = Nothing
        , tracker = Nothing
        }


type alias CreateResponse =
    { error : Bool
    , message : String
    }


createDecoder : Decoder CreateResponse
createDecoder =
    map2 CreateResponse
        (field "error" bool)
        (field "message" string)
