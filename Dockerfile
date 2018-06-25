# |-------<[ Build ]>-------|

FROM ruby:2.5-alpine AS build

RUN mkdir -p /build
WORKDIR /build

RUN apk add --no-cache \
    build-base

COPY Gemfile Gemfile.lock ./
RUN bundle install --deployment --without="development test"


# |-------<[ App ]>-------|

FROM ruby:2.5-alpine AS app

LABEL maintainer="Patrick Auernig <dev.patrick.auernig@gmail.com>"

RUN apk add --no-cache \
    tzdata

ARG user_uid=1000
ARG user_gid=1000
RUN addgroup -S -g "$user_gid" app \
 && adduser -S -G app -u "$user_uid" app

RUN mkdir -p /app && chown app:app /app
WORKDIR /app
USER app

COPY --chown=app:app --from=build /build/vendor/bundle ./vendor/bundle
COPY Gemfile Gemfile.lock ./
RUN bundle install --deployment --without="development test"
COPY --chown=app:app ./ ./

EXPOSE 9393
ENTRYPOINT ["bundle", "exec"]
CMD ["unicorn", "--config-file", "unicorn.rb"]
