FROM node:20-slim AS builder

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

WORKDIR /app/frontend

COPY package.json pnpm-lock.yaml* pnpm-workspace.yaml .npmrc* ./
RUN pnpm install

COPY --from=root openapi.yaml ../
COPY . .
RUN pnpm build

FROM nginx:alpine
COPY --from=builder /app/frontend/dist /var/www/textury
COPY --from=root nginx.prod.conf /etc/nginx/conf.d/default.conf

EXPOSE 8001

CMD ["nginx", "-g", "daemon off;"]