import { useCallback, useSyncExternalStore } from 'react'
import {
  FaChevronDown,
  FaClipboard,
  FaExternalLinkAlt,
  FaGithub,
  FaMoon,
  FaSun,
} from 'react-icons/fa'
import {
  Box,
  Button,
  Dialog,
  Flex,
  HStack,
  Heading,
  Icon,
  IconButton,
  Input,
  Link,
  Menu,
  Popover,
  Portal,
  Text,
  VStack,
  useClipboard,
  useDisclosure,
} from '@chakra-ui/react'
import { usePathname, useSearchParams } from 'next/navigation'

import { Example, examples } from '@/examples'
import { useColorMode, useColorModeValue } from '@/hooks/use-color-mode'

export type HeaderMenuProps = {
  height: string
  setCodeToLoad: (code: string) => void
}

export function HeaderMenu(props: HeaderMenuProps) {
  const { height, setCodeToLoad } = props

  return (
    <Flex
      height={height}
      flexDirection="row"
      alignItems="center"
      width="100%"
      bg="ctp.crust"
      paddingX={1}
    >
      <HStack gap={4} alignItems="center">
        <Heading as="h1" size={{ base: '2xl', md: '3xl' }} lineHeight="normal" color="ctp.text">
          Rimu
        </Heading>
        <ExamplesMenu setCodeToLoad={setCodeToLoad} />
        <HelpButton />
        <SharePopover />
      </HStack>

      <Box flexGrow={1} />

      <Flex flexDirection="row" alignItems="center">
        <GitHubSourceButton />
        <ColorModeSwitch />
      </Flex>
    </Flex>
  )
}

type ExamplesMenuProps = {
  setCodeToLoad: (code: string) => void
}

function ExamplesMenu(props: ExamplesMenuProps) {
  const { setCodeToLoad } = props

  return (
    <Menu.Root>
      <Menu.Trigger asChild>
        <Button
          bg="ctp"
          color="ctp.text"
          fontWeight="bold"
          _hover={{ bg: 'ctp.mantle' }}
          size={{ base: 'xs', md: 'sm' }}
        >
          Examples
          <Icon marginStart={2}>
            <FaChevronDown />
          </Icon>
        </Button>
      </Menu.Trigger>
      <Portal>
        <Menu.Positioner>
          <Menu.Content>
            {examples.map((example: Example, index: number) => {
              const { name, code } = example
              return (
                <Menu.Item key={index} value={name} onClick={() => setCodeToLoad(code)}>
                  {name}
                </Menu.Item>
              )
            })}
          </Menu.Content>
        </Menu.Positioner>
      </Portal>
    </Menu.Root>
  )
}

function HelpButton() {
  const { open, onOpen, onClose } = useDisclosure()

  return (
    <Dialog.Root open={open} onOpenChange={(d) => (d.open ? onOpen() : onClose())}>
      <Dialog.Trigger asChild>
        <Button
          bg="ctp"
          color="ctp.text"
          fontWeight="bold"
          _hover={{ bg: 'ctp.mantle' }}
          size={{ base: 'xs', md: 'sm' }}
        >
          Help
        </Button>
      </Dialog.Trigger>
      <Portal>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content>
            <Dialog.Header>
              <Dialog.Title>Help</Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              <Text>
                Learn about Rimu:{' '}
                <Link
                  href="https://rimu.dev"
                  color="ctp.teal"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  rimu.dev
                  <Icon marginX={1}>
                    <FaExternalLinkAlt />
                  </Icon>
                </Link>
              </Text>
            </Dialog.Body>
            <Dialog.Footer>
              <Dialog.CloseTrigger asChild>
                <Button bg="ctp.teal" color="ctp" _hover={{ bg: 'ctp.sky' }} mr={3}>
                  Close
                </Button>
              </Dialog.CloseTrigger>
            </Dialog.Footer>
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  )
}

function SharePopover() {
  const origin = useSyncExternalStore(
    () => () => {},
    () => window.location.origin,
    () => '',
  )
  const pathname = usePathname()
  const searchParams = useSearchParams()
  const url =
    Array.from(searchParams.keys()).length === 0
      ? `${origin}${pathname}`
      : `${origin}${pathname}?${searchParams}`

  const clipboard = useClipboard({ value: url })
  const onCopy = useCallback(() => clipboard.copy(), [clipboard])

  return (
    <Popover.Root>
      <Popover.Trigger asChild>
        <Button
          bg="ctp"
          color="ctp.text"
          fontWeight="bold"
          _hover={{ bg: 'ctp.mantle' }}
          size={{ base: 'xs', md: 'sm' }}
        >
          Share
        </Button>
      </Popover.Trigger>
      <Portal>
        <Popover.Positioner>
          <Popover.Content>
            <Popover.Arrow />
            <Popover.CloseTrigger />
            <Popover.Header>Share your code!</Popover.Header>
            <Popover.Body>
              <VStack>
                <HStack gap={3} alignSelf="stretch">
                  <Text as="label" minWidth="3rem">
                    URL:
                  </Text>
                  <Input
                    type="text"
                    variant="outline"
                    value={url}
                    readOnly
                    width="100%"
                    borderRadius={4}
                    padding={1}
                  />
                </HStack>
                <Button
                  alignSelf="center"
                  onClick={onCopy}
                  bg="ctp.teal"
                  color="ctp"
                  _hover={{ bg: 'ctp.sky' }}
                >
                  Copy to clipboard
                  <Icon marginStart={2}>
                    <FaClipboard />
                  </Icon>
                </Button>
              </VStack>
            </Popover.Body>
          </Popover.Content>
        </Popover.Positioner>
      </Portal>
    </Popover.Root>
  )
}

function GitHubSourceButton() {
  return (
    <Link href="https://github.com/ahdinosaur/rimu" target="_blank" rel="noopener noreferrer">
      <IconButton aria-label="GitHub source" variant="ghost">
        <Icon>
          <FaGithub />
        </Icon>
      </IconButton>
    </Link>
  )
}

function ColorModeSwitch() {
  const { toggleColorMode } = useColorMode()
  const text = useColorModeValue('dark', 'light')
  const switchIcon = useColorModeValue(<FaMoon />, <FaSun />)

  return (
    <IconButton
      fontSize="lg"
      aria-label={`Switch to ${text} mode`}
      variant="ghost"
      color="ctp.text"
      onClick={toggleColorMode}
    >
      <Icon>{switchIcon}</Icon>
    </IconButton>
  )
}
