import { FaMoon, FaSun } from 'react-icons/fa'
import { Box, Flex, Heading, IconButton, useColorMode, useColorModeValue } from '@chakra-ui/react'

export type HeaderMenuProps = {}

export function HeaderMenu(_props: HeaderMenuProps) {
  return (
    <Flex sx={{ flexDirection: 'row', width: '100%', backgroundColor: 'rimu-header-bg' }}>
      <Box>
        <Heading as="h1" sx={{ alignSelf: 'start', paddingLeft: 4 }}>
          Rimu
        </Heading>
      </Box>

      <Box sx={{ flexGrow: 1 }} />

      <Flex sx={{ flexDirection: 'row' }}>
        <ColorModeSwitch />
      </Flex>
    </Flex>
  )
}

function ColorModeSwitch() {
  const { toggleColorMode } = useColorMode()

  const text = useColorModeValue('dark', 'light')
  const SwitchIcon = useColorModeValue(FaMoon, FaSun)

  return (
    <IconButton
      size="md"
      fontSize="lg"
      aria-label={`Switch to ${text} mode`}
      variant="ghost"
      color="current"
      onClick={toggleColorMode}
      icon={<SwitchIcon />}
    />
  )
}