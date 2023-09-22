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
  Flex,
  HStack,
  Heading,
  IconButton,
  Menu,
  MenuButton,
  MenuItem,
  MenuList,
  Modal,
  ModalOverlay,
  ModalContent,
  ModalHeader,
  ModalFooter,
  ModalBody,
  ModalCloseButton,
  useColorMode,
  useColorModeValue,
  useDisclosure,
  Text,
  Link,
  Icon,
  Popover,
  PopoverTrigger,
  PopoverContent,
  PopoverArrow,
  PopoverCloseButton,
  PopoverHeader,
  PopoverBody,
  Input,
  FormLabel,
  VStack,
} from '@chakra-ui/react'
import { useEffect, useState } from 'react'
import { useClipboard } from 'use-clipboard-copy'
import { usePathname, useSearchParams } from 'next/navigation'

import { Example, examples } from '@/examples'

export type HeaderMenuProps = {
  height: string
  setCodeToLoad: (code: string) => void
}

export function HeaderMenu(props: HeaderMenuProps) {
  const { height, setCodeToLoad } = props

  return (
    <Flex
      sx={{
        height,
        flexDirection: 'row',
        alignItems: 'center',
        width: '100%',
        backgroundColor: 'rimu.header.background',
        paddingX: 1,
      }}
    >
      <HStack spacing={4} sx={{ alignItems: 'center' }}>
        <Heading as="h1" size="lg" sx={{ lineHeight: 'normal' }}>
          Rimu
        </Heading>
        <ExamplesMenu setCodeToLoad={setCodeToLoad} />
        <HelpButton />
        <SharePopover />
      </HStack>

      <Box sx={{ flexGrow: 1 }} />

      <Flex sx={{ flexDirection: 'row', alignItems: 'center' }}>
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
    <Menu>
      <MenuButton as={Button} size="sm" rightIcon={<FaChevronDown />}>
        Examples
      </MenuButton>
      <MenuList>
        {examples.map((example, index) => {
          const { name, code } = example
          return (
            <MenuItem key={index} onClick={() => setCodeToLoad(code)}>
              {name}
            </MenuItem>
          )
        })}
      </MenuList>
    </Menu>
  )
}

function HelpButton() {
  const { isOpen, onOpen, onClose } = useDisclosure()

  return (
    <>
      <Button size="sm" onClick={onOpen}>
        Help
      </Button>

      <Modal isOpen={isOpen} onClose={onClose}>
        <ModalOverlay />
        <ModalContent>
          <ModalHeader>Help</ModalHeader>
          <ModalCloseButton />
          <ModalBody>
            <Text>
              <Link
                href="https://rimu.dev"
                sx={{ color: { _light: 'teal.600', _dark: 'teal.50' } }}
                isExternal
              >
                rimu.dev
                <Icon sx={{ marginX: 1 }} as={FaExternalLinkAlt} />
              </Link>
            </Text>

            <Text></Text>
          </ModalBody>

          <ModalFooter>
            <Button colorScheme="teal" mr={3} onClick={onClose}>
              Close
            </Button>
          </ModalFooter>
        </ModalContent>
      </Modal>
    </>
  )
}

function SharePopover() {
  const [origin, setOrigin] = useState('')
  const pathname = usePathname()
  const searchParams = useSearchParams()
  const url =
    Array.from(searchParams.keys()).length === 0
      ? `${origin}${pathname}`
      : `${origin}${pathname}?${searchParams}`

  useEffect(() => {
    setOrigin(location.origin)
  }, [])

  const clipboard = useClipboard()

  return (
    <Popover>
      <PopoverTrigger>
        <Button size="sm">Share</Button>
      </PopoverTrigger>
      <PopoverContent>
        <PopoverArrow />
        <PopoverCloseButton />
        <PopoverHeader>Share your code!</PopoverHeader>
        <PopoverBody>
          <VStack>
            <HStack spacing={3} sx={{ alignSelf: 'stretch' }}>
              <FormLabel>URL:</FormLabel>
              <Input
                ref={clipboard.target}
                type="text"
                variant="outline"
                value={url}
                readOnly
                sx={{ width: '100%', borderRadius: 4, padding: 1 }}
              />
            </HStack>
            <Button
              rightIcon={<Icon as={FaClipboard} />}
              sx={{ alignSelf: 'center' }}
              onClick={clipboard.copy}
            >
              Copy to clipboard
            </Button>
          </VStack>
        </PopoverBody>
      </PopoverContent>
    </Popover>
  )
}

function GitHubSourceButton() {
  return (
    <Link href="https://github.com/ahdinosaur/rimu">
      <IconButton aria-label="GitHub source" icon={<Icon as={FaGithub} />} variant="ghost" />
    </Link>
  )
}

function ColorModeSwitch() {
  const { toggleColorMode } = useColorMode()

  const text = useColorModeValue('dark', 'light')
  const SwitchIcon = useColorModeValue(FaMoon, FaSun)

  return (
    <IconButton
      fontSize="lg"
      aria-label={`Switch to ${text} mode`}
      variant="ghost"
      color="current"
      onClick={toggleColorMode}
      icon={<SwitchIcon />}
    />
  )
}
