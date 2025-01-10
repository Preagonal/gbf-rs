import { Button, Container, Text, Title } from '@mantine/core';

export default function Home() {
  return (
    <Container size="lg" my="xl">
      <Title order={1}>Welcome to GBF Web</Title>
      <Text mt="sm">
        This is a Next.js application styled with Mantine, built to track decompiler and CFG progress.
      </Text>
      <Button mt="lg" variant="outline">
        Learn More
      </Button>
    </Container>
  );
}
