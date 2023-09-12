import { FaMoon, FaSun } from 'react-icons/fa'
import { Box, Flex, Heading, IconButton, useColorMode, useColorModeValue } from '@chakra-ui/react'

export type HeaderMenuProps = {
  height: string
}

export function HeaderMenu(props: HeaderMenuProps) {
  const { height } = props

  return (
    <Flex
      sx={{
        height,
        flexDirection: 'row',
        alignItems: 'baseline',
        width: '100%',
        backgroundColor: 'rimu.header.background',
      }}
    >
      <Flex sx={{ flexDirection: 'row', alignItems: 'baseline' }}>
        <Heading
          as="h1"
          size="lg"
          sx={{ alignSelf: 'start', paddingLeft: 4, lineHeight: 'normal' }}
        >
          Rimu
        </Heading>
      </Flex>

      <Box sx={{ flexGrow: 1 }} />

      <Flex sx={{ flexDirection: 'row', alignItems: 'baseline' }}>
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
      size="sm"
      fontSize="lg"
      aria-label={`Switch to ${text} mode`}
      variant="ghost"
      color="current"
      onClick={toggleColorMode}
      icon={<SwitchIcon />}
    />
  )
}
